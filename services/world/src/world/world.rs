use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use azerust_game::{
    accounts::AccountService,
    characters::{AccountData, CharacterCreate, CharacterService},
    realms::{RealmId, RealmList},
};
use azerust_protocol::{world::ResponseCode, Addon, ClientPacket, Item, ServerPacket};
use tokio::{
    join,
    net::tcp::OwnedWriteHalf,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
        Mutex, RwLock,
    },
    time::{interval, Interval},
};
use tracing::{error, trace};

use super::Session;
use crate::client::{Client, ClientId};

pub const GLOBAL_CACHE_MASK: u32 = 0x15;

pub struct World<A: AccountService, R: RealmList, C: CharacterService> {
    id: RealmId,
    accounts: A,
    realms: R,
    characters: C,
    receiver: Mutex<Receiver<(ClientId, ClientPacket)>>,
    sender: Sender<(ClientId, ClientPacket)>,
    sessions: Arc<RwLock<HashMap<ClientId, Arc<Session>>>>,

    start: SystemTime,
}

impl<A: AccountService, R: RealmList, C: CharacterService> World<A, R, C> {
    pub fn new(id: RealmId, accounts: A, realms: R, characters: C) -> Self {
        let (sender, receiver) = unbounded_channel();
        Self {
            id,
            accounts,
            realms,
            characters,
            sender,
            receiver: Mutex::new(receiver),
            sessions: Default::default(),

            start: SystemTime::now(),
        }
    }

    /// runs background tasks
    pub async fn timers(&self) -> Result<()> {
        let mut timers = WorldTimers::new();

        let uptime = async {
            loop {
                timers.uptime.tick().await;
                if let Err(e) = self.realms.set_uptime(self.id, self.start, 0).await {
                    error!("error when setting uptime: {e}");
                }
            }
        };

        let ping_db = async {
            loop {
                timers.ping_db.tick().await;
            }
        };

        join!(ping_db, uptime);
        // todo(arlyon): update uptime and population
        // todo(arlyon): ping database

        Ok(())
    }

    pub async fn handle_packets(&self) -> Result<()> {
        let mut receiver = self.receiver.lock().await;
        loop {
            trace!("waiting for packet");
            let (id, packet) = receiver.recv().await.ok_or_else(|| anyhow!("no packet"))?;
            trace!("getting session");
            let session = {
                let sessions = self.sessions.read().await;
                match sessions.get(&id).cloned() {
                    Some(c) => c,
                    None => continue,
                }
            };

            trace!("handling packet");
            if self.handle_packet(session, packet).await.is_err() {
                error!("could not handle packet from client {:?}", id);
            }
            trace!("handled!");
        }
    }

    async fn handle_packet(&self, session: Arc<Session>, packet: ClientPacket) -> Result<()> {
        match packet {
            ClientPacket::AuthSession(_) => Ok(()), // ignore
            ClientPacket::KeepAlive => session.reset_timeout().await,
            ClientPacket::Ping { latency, seq } => {
                session.set_latency(latency).await;
                session.send_packet(ServerPacket::Pong(seq)).await
            }

            ClientPacket::ReadyForAccountDataTimes => {
                let data = match session.client.read().await.account {
                    Some(id) => self
                        .characters
                        .account_data(id)
                        .await
                        .map_err(|_| anyhow!("unable to get character account data"))?,
                    None => AccountData::default(),
                };

                session
                    .send_packet(ServerPacket::AccountDataTimes(Box::new(data)))
                    .await
            }
            ClientPacket::CharEnum => {
                let id = session
                    .client
                    .read()
                    .await
                    .account
                    .ok_or_else(|| anyhow!("no account"))?;
                let characters = self
                    .characters
                    .get_by_account(id)
                    .await
                    .map_err(|_| anyhow!("unable to get character list"))?;
                let items = [Item {
                    display: 0,
                    inventory: 0,
                    aura: 0,
                }; 23];
                session
                    .send_packet(ServerPacket::CharEnum(
                        characters.into_iter().map(|c| (c, items)).collect(),
                    ))
                    .await
            }
            ClientPacket::RealmSplit { realm } => {
                session
                    .send_packet(ServerPacket::RealmSplit { realm })
                    .await
            }
            ClientPacket::CharacterCreate {
                name,
                race,
                class,
                gender,
                skin_color,
                face,
                hair_style,
                hair_color,
                facial_style,
            } => {
                let available = self.characters.name_available(&name).await.unwrap_or(false);
                if !available {
                    return session
                        .send_packet(ServerPacket::CharacterCreate(
                            ResponseCode::CharCreateNameInUse,
                        ))
                        .await;
                }

                let mut name_proper = name.to_ascii_lowercase();
                if let Some(r) = name_proper.get_mut(0..1) {
                    r.make_ascii_uppercase();
                }

                if let Some(x) = match &name {
                    n if name_proper.ne(n) => Some(ResponseCode::CharNameFailure),
                    n if n.is_empty() => Some(ResponseCode::CharNameNoName),
                    n if n.len() < 2 => Some(ResponseCode::CharNameTooShort),
                    _ => None,
                } {
                    return session.send_packet(ServerPacket::CharacterCreate(x)).await;
                }

                self.characters
                    .create_character(
                        session
                            .client
                            .read()
                            .await
                            .account
                            .ok_or_else(|| anyhow!("no account"))?,
                        CharacterCreate {
                            name,
                            race,
                            class,
                            gender,
                            skin_color,
                            face,
                            hair_style,
                            hair_color,
                            facial_style,
                            map: 0,               //
                            zone: 1,              //
                            position_x: -6240.32, // dwarf start zone
                            position_y: 331.033,  //
                            position_z: 382.758,  //
                        },
                    )
                    .await
                    .map_err(|_| anyhow!("unable to create character"))?;

                session
                    .send_packet(ServerPacket::CharacterCreate(
                        ResponseCode::CharCreateSuccess,
                    ))
                    .await
            }
            ClientPacket::PlayerLogin(id) => {
                let character = self
                    .characters
                    .get(id.try_into()?)
                    .await
                    .context("unable to get character list")?;
                session.login(character).await
            }
            ClientPacket::CharacterDelete(id) => match self
                .characters
                .delete_character(id.try_into().context("invalid guid provided")?)
                .await
                .context("unable to delete character")
            {
                Ok(_) => {
                    session
                        .send_packet(ServerPacket::CharacterDelete(
                            ResponseCode::CharDeleteSuccess,
                        ))
                        .await
                }
                Err(_) => {
                    error!("failed to delete character: {:?}", id);
                    session
                        .send_packet(ServerPacket::CharacterDelete(
                            ResponseCode::CharDeleteFailed,
                        ))
                        .await
                }
            },
        }
    }

    /// updates the world
    pub async fn update(&self, _diff: Duration) {
        // update game time
    }

    pub async fn create_session(
        &self,
        client: Arc<RwLock<Client>>,
        writer: OwnedWriteHalf,
        session_key: [u8; 40],
        addons: Vec<Addon>,
    ) -> Result<Arc<Session>, (anyhow::Error, OwnedWriteHalf)> {
        let session = Arc::new(
            match Session::new(client, writer, session_key, self.sender.clone(), addons).await {
                Ok(s) => s,
                Err((e, w)) => return Err((e, w)),
            },
        );
        self.sessions
            .write()
            .await
            .insert(session.client_id, session.clone());
        Ok(session)
    }
}

struct WorldTimers {
    uptime: Interval,
    ping_db: Interval,
}

impl WorldTimers {
    fn new() -> Self {
        Self {
            uptime: interval(Duration::from_secs(60)),
            ping_db: interval(Duration::from_secs(60 * 10)),
        }
    }
}
