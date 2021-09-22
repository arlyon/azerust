use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::TcpStream,
    prelude::*,
    stream::{interval, Interval},
    sync::RwLock,
};
use azerust_game::{
    accounts::AccountService,
    characters::{AccountData, CharacterCreate, CharacterService},
    realms::{RealmId, RealmList},
};
use azerust_protocol::{world::ResponseCode, Addon, ClientPacket, Item, ServerPacket};
use tracing::error;

use super::Session;
use crate::client::{Client, ClientId};

pub const GLOBAL_CACHE_MASK: u32 = 0x15;

pub struct World<A: AccountService, R: RealmList, C: CharacterService> {
    id: RealmId,
    accounts: A,
    realms: R,
    characters: C,
    receiver: Receiver<(ClientId, ClientPacket)>,
    sender: Sender<(ClientId, ClientPacket)>,
    sessions: Arc<RwLock<HashMap<ClientId, Arc<Session>>>>,

    start: SystemTime,
}

impl<A: AccountService, R: RealmList, C: CharacterService> World<A, R, C> {
    pub fn new(id: RealmId, accounts: A, realms: R, characters: C) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            id,
            accounts,
            realms,
            characters,
            sender,
            receiver,
            sessions: Default::default(),

            start: SystemTime::now(),
        }
    }

    /// runs background tasks
    pub async fn timers(&self) -> Result<()> {
        let mut timers = WorldTimers::new();

        let uptime = async {
            while timers.uptime.next().await.is_some() {
                if let Err(e) = self.realms.set_uptime(self.id, self.start, 0).await {
                    error!("error when setting uptime: {}", e);
                }
            }
        };

        let ping_db = async { while timers.ping_db.next().await.is_some() {} };

        uptime.join(ping_db).await;
        // todo(arlyon): update uptime and population
        // todo(arlyon): ping database

        Ok(())
    }

    pub async fn handle_packets(&self) -> Result<()> {
        loop {
            let (id, packet) = self.receiver.recv().await?;
            let session = {
                let sessions = self.sessions.read().await;
                match sessions.get(&id).cloned() {
                    Some(c) => c,
                    None => continue,
                }
            };

            if self.handle_packet(session, packet).await.is_err() {
                error!("could not handle packet from client {:?}", id);
            }
        }
    }

    pub async fn handle_packet(&self, session: Arc<Session>, packet: ClientPacket) -> Result<()> {
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
                let id = session.client.read().await.account.unwrap();
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
                        session.client.read().await.account.unwrap(),
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
        stream: TcpStream,
        session_key: [u8; 40],
        addons: Vec<Addon>,
    ) -> Result<Arc<Session>> {
        let session =
            Arc::new(Session::new(client, stream, session_key, self.sender.clone(), addons).await?);
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
