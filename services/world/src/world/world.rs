use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, SystemTime},
};

use anyhow::{anyhow, Context, Result};
use azerust_game::{
    accounts::AccountService,
    characters::{AccountData, AccountDataType, CharacterCreate, CharacterService},
    realms::{RealmId, RealmList},
};
use azerust_protocol::world::{
    Addon, ClientPacket, Item, Position, RelationType, ResponseCode, ServerPacket, VerifyWorld,
};
use opentelemetry::{
    trace::{SpanContext, TraceContextExt},
    Context as OpentelemetryContext,
};
use tokio::{
    join,
    net::tcp::OwnedWriteHalf,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver as Receiver, UnboundedSender as Sender},
        Mutex, RwLock,
    },
    time::{interval, Interval},
};
use tracing::{error, info, instrument, span, trace, warn, Level};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::Session;
use crate::client::{Client, ClientId};

pub struct World<A: AccountService, R: RealmList, C: CharacterService> {
    id: RealmId,
    accounts: A,
    realms: R,
    characters: C,
    receiver: Mutex<Receiver<(ClientId, ClientPacket, OpentelemetryContext)>>,
    sender: Sender<(ClientId, ClientPacket, OpentelemetryContext)>,
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
    #[instrument(skip(self))]
    pub async fn timers(&self) -> Result<()> {
        let mut timers = WorldTimers::new();

        let uptime = async {
            loop {
                timers.uptime.tick().await;
                // todo(arlyon): set population
                if let Err(e) = self.realms.set_uptime(self.id, self.start, 0).await {
                    error!("error when setting uptime: {e}");
                }
            }
        };

        let ping_db = async {
            loop {
                // todo(arlyon): ping database
                timers.ping_db.tick().await;
            }
        };

        join!(ping_db, uptime);

        Ok(())
    }

    pub async fn handle_packets(&self) -> Result<()> {
        let mut receiver = self.receiver.lock().await;
        loop {
            let (id, packet, ctx) = receiver.recv().await.ok_or_else(|| anyhow!("no packet"))?;
            let span = span!(
                Level::TRACE,
                "handle_packets",
                "client_id" = id.0,
                "otel.kind" = "CONSUMER",
            );
            span.set_parent(ctx);
            let _guard = span.enter();

            let session = {
                let sessions = self.sessions.read().await;
                match sessions.get(&id).cloned() {
                    Some(c) => c,
                    None => continue,
                }
            };

            if let Err(e) = self.handle_packet(session, packet).await {
                error!("could not handle packet from client {id:?}: {e}");
            }
        }
    }

    #[instrument(skip(self, session))]
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
                    .send_packet(ServerPacket::AccountDataTimes(
                        Box::new(data),
                        AccountDataType::Account,
                    ))
                    .await
                    .context("couldn't write")
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

                let name = character.name.clone();

                trace!("loggin in {name}");

                let verify_world = VerifyWorld {
                    map_id: character.map.into(),
                    position: Position {
                        x: character.position_x,
                        y: character.position_y,
                        z: character.position_z,
                        orientation: 0.0, // todo(arlyon): orientation
                    },
                };

                let _old_char = session.login(character).await?;

                // verify world
                {
                    session
                        .send_packet(ServerPacket::VerifyWorld(verify_world))
                        .await?;
                }

                // account data times
                {
                    let data = self
                        .characters
                        .account_data(session.client.read().await.account.expect("account"))
                        .await?;

                    session
                        .send_packet(ServerPacket::AccountDataTimes(
                            Box::new(data),
                            AccountDataType::PerChar,
                        ))
                        .await?;
                }

                session
                    .send_packet(ServerPacket::FeatureSystemStatus)
                    .await?;

                session
                    .send_packet(ServerPacket::Motd("Hello World".to_string()))
                    .await?;

                session
                    .send_packet(ServerPacket::SysMessage(
                        env!("CARGO_PKG_VERSION").to_string(),
                    ))
                    .await?;

                session.send_packet(ServerPacket::LearnedDanceMoves).await?;

                {
                    // pre-map packets

                    session.send_packet(ServerPacket::SocialList).await?;

                    session
                        .send_packet(ServerPacket::BindPointUpdate {
                            x: -6240.32,
                            y: 331.033,
                            z: 382.758,
                            map: 0,
                            area: 1,
                        })
                        .await?;
                    session.send_packet(ServerPacket::TutorialData).await?;

                    session.send_packet(ServerPacket::TalentsInfo).await?;
                    session
                        .send_packet(ServerPacket::InstanceDifficulty)
                        .await?;
                    session.send_packet(ServerPacket::InitialSpells).await?;
                    session.send_packet(ServerPacket::SendUnlearnSpells).await?;
                    session
                        .send_packet(ServerPacket::InitialActionButtons)
                        .await?;

                    session
                        .send_packet(ServerPacket::InitializeFactions)
                        .await?;
                    session
                        .send_packet(ServerPacket::AllAchievementData)
                        .await?;
                    session
                        .send_packet(ServerPacket::ContactList(vec![
                            RelationType::Friend,
                            RelationType::Muted,
                            RelationType::Ignore,
                        ]))
                        .await?;
                    session.send_packet(ServerPacket::AuraUpdateAll).await?;
                    session.send_packet(ServerPacket::EquipmentList).await?;
                    session.send_packet(ServerPacket::LoginSetTimeSpeed).await?;
                    session
                        .send_packet(ServerPacket::SetForcedReactions)
                        .await?;
                }

                // show cinematic
                if false {
                    session.send_packet(ServerPacket::CinematicStart(1)).await?;
                    // todo(arlyon): cinematic manager

                    session
                        .send_packet(ServerPacket::SysMessage(
                            "New character created!".to_string(),
                        ))
                        .await?;
                }

                // add player to map

                {
                    // send packets after adding to map
                    session
                        .send_packet(ServerPacket::InitialWorldStates(
                            session.character.read().await.clone().unwrap(),
                        ))
                        .await?;

                    session.send_packet(ServerPacket::SendTimeSync(0)).await?;

                    // session.cast_spell(836, true); // LOGINEFFECT
                }

                // set char status online
                // set login status online

                info!(
                    "account {} logged in with character {name} ({})",
                    "arlyon", "127.0.0.1"
                );

                Ok(())
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
            ClientPacket::SetActiveVoiceChannel(_) => Ok(()), // todo(arlyon): implement
            ClientPacket::NameQuery(id) => match self.characters.get(id.try_into()?).await {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            },
            ClientPacket::UpdateAccountData(d) => {
                // todo(arlyon): update
                warn!("received account data update");
                Ok(())
            }
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
