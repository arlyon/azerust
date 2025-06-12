use std::{
    collections::HashMap,
    convert::TryInto,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use azerust_game::{
    Guid,
    accounts::AccountService,
    characters::{CharacterCreate, CharacterId, CharacterService},
    realms::{RealmId, RealmList},
};
use azerust_utils::flatten;
use rand::Rng;
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket, tcp::OwnedWriteHalf},
    sync::{Mutex, RwLock},
    time::interval,
    try_join,
};
use tokio_stream::{StreamExt, wrappers::TcpListenerStream};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, instrument, warn};
use wow_srp::{normalized_string::NormalizedString, wrath_header::ProofSeed};
use wow_world_messages::{
    errors::ExpectedOpcodeError,
    wrath::{
        ActionButton, Area, ArenaType, BattlegroundType, BillingPlanFlags, CMSG_AUTH_SESSION, CacheMask, ClientMessage, ComplaintStatus, CreatureFamily,
        DungeonDifficulty, Expansion, FactionInitializer, Level, LogoutResult, LogoutSpeed, MSG_MOVE_FALL_LAND, MSG_MOVE_HEARTBEAT, MSG_MOVE_JUMP,
        MSG_MOVE_SET_FACING, MSG_MOVE_SET_RUN_MODE, MSG_MOVE_SET_WALK_MODE, MSG_MOVE_START_BACKWARD, MSG_MOVE_START_FORWARD, MSG_MOVE_START_PITCH_DOWN,
        MSG_MOVE_START_PITCH_UP, MSG_MOVE_START_STRAFE_LEFT, MSG_MOVE_START_STRAFE_RIGHT, MSG_MOVE_START_SWIM, MSG_MOVE_START_TURN_LEFT,
        MSG_MOVE_START_TURN_RIGHT, MSG_MOVE_STOP, MSG_MOVE_STOP_PITCH, MSG_MOVE_STOP_STRAFE, MSG_MOVE_STOP_SWIM, MSG_MOVE_STOP_TURN,
        MSG_SET_DUNGEON_DIFFICULTY_Server, MovementBlock, MovementBlock_MovementFlags, MovementBlock_UpdateFlag, MovementBlock_UpdateFlag_Living, MovementInfo,
        Object, ObjectType, Power, RealmSplitState, Relation, Relation_RelationType, RelationType, SMSG_ACCOUNT_DATA_TIMES, SMSG_ACTION_BUTTONS,
        SMSG_AUTH_CHALLENGE, SMSG_AUTH_RESPONSE, SMSG_BATTLEFIELD_STATUS, SMSG_BATTLEFIELD_STATUS_StatusId, SMSG_BINDPOINTUPDATE, SMSG_CHAR_CREATE,
        SMSG_CHAR_ENUM, SMSG_CONTACT_LIST, SMSG_FEATURE_SYSTEM_STATUS, SMSG_FRIEND_STATUS, SMSG_GMTICKET_GETTICKET, SMSG_INIT_WORLD_STATES,
        SMSG_INITIALIZE_FACTIONS, SMSG_LOGIN_SETTIMESPEED, SMSG_LOGIN_VERIFY_WORLD, SMSG_LOGOUT_COMPLETE, SMSG_LOGOUT_RESPONSE, SMSG_MESSAGECHAT,
        SMSG_MESSAGECHAT_ChatType, SMSG_NAME_QUERY_RESPONSE, SMSG_PLAYED_TIME, SMSG_PONG, SMSG_QUERY_TIME_RESPONSE, SMSG_REALM_SPLIT, SMSG_TIME_SYNC_REQ,
        SMSG_TUTORIAL_FLAGS, SMSG_UPDATE_OBJECT, SMSG_WORLD_STATE_UI_TIMER_UPDATE, ServerMessage, UpdateMask, UpdatePlayer, Vector3d, WorldResult, WorldState,
        opcodes::ClientOpcodeMessage, tokio_expect_client_message,
    },
};

use crate::{
    client::{Client, ClientId},
    world::World,
};

#[allow(unused)]
pub struct WorldServer<A: AccountService + Sync, R: RealmList + Sync, C: CharacterService + Sync> {
    id: RealmId,
    accounts: A,
    realms: R,
    characters: C,
    auth_server_address: String,
    world: World<A, R, C, OwnedWriteHalf>,

    // TODO: use
    realm_seed: [u8; 4],
    clients: RwLock<HashMap<ClientId, Arc<RwLock<Client>>>>,
    running: AtomicBool,
}

impl<A: AccountService + Sync + Send + 'static + Clone, R: RealmList + Sync + Send + 'static + Clone, C: CharacterService + Sync + Send + 'static + Clone>
    WorldServer<A, R, C>
{
    pub fn new(realm_id: RealmId, accounts: A, realms: R, characters: C, auth_server_address: String) -> Self {
        WorldServer::with_world(
            realm_id,
            accounts.clone(),
            realms.clone(),
            characters.clone(),
            World::new(RealmId(0), accounts, realms, characters),
            auth_server_address,
        )
    }

    pub fn with_world(realm_id: RealmId, accounts: A, realms: R, characters: C, world: World<A, R, C, OwnedWriteHalf>, auth_server_address: String) -> Self {
        Self {
            world,
            accounts,
            realms,
            characters,
            auth_server_address,
            id: realm_id,
            realm_seed: rand::thread_rng().r#gen(),
            clients: Default::default(),

            running: AtomicBool::new(true),
        }
    }

    /// Sends periodic heartbeat packets to the auth server
    #[instrument(skip(self))]
    pub async fn auth_server_heartbeat(&self) -> Result<()> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        socket.connect(&self.auth_server_address).await?;

        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            let [a, b, c, d] = 0u32.to_be_bytes();
            if let Err(_e) = socket.send(&[0u8, self.id.0 as u8, a, b, c, d]).await {
                warn!("could not send heartbeat to {}", self.auth_server_address);
            }
        }
    }

    /// Allows the world server to accept new clients
    #[instrument(skip(self))]
    pub async fn accept_clients(self: Arc<Self>) -> Result<()> {
        let addr = ("0.0.0.0", 8085);
        let listener = TcpListener::bind(&addr).await?;

        info!("listening on {:?}", &addr);

        let mut connections = TcpListenerStream::new(listener).filter_map(|s| s.ok());
        while let Some(stream) = connections.next().await {
            tokio::task::Builder::new()
                .name(&format!(
                    "world::connect[{}]",
                    stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".to_string())
                ))
                .spawn({
                    let server = self.to_owned();
                    async move {
                        if let Err(e) = server.connect_loop(stream).await {
                            error!("error handling request: {e}")
                        }
                    }
                })?;
        }

        Ok(())
    }

    async fn connect_loop(self: Arc<Self>, mut stream: TcpStream) -> Result<()> {
        let seed = ProofSeed::new();

        SMSG_AUTH_CHALLENGE {
            unknown1: 0,
            server_seed: seed.seed(),
            seed: [0; 32],
        }
        .tokio_write_unencrypted_server(&mut stream)
        .await
        .unwrap();

        let c = tokio_expect_client_message::<CMSG_AUTH_SESSION, _>(&mut stream).await.unwrap();

        let account = self.accounts.get_by_username(&c.username).await.unwrap().unwrap();

        let session_key = account.session_key.unwrap();

        let (mut encryption, decryption) = seed
            .into_server_header_crypto(&NormalizedString::new(&c.username).unwrap(), session_key, c.client_proof, c.client_seed)
            .unwrap()
            .split();

        SMSG_AUTH_RESPONSE::AuthOk {
            billing_flags: BillingPlanFlags::empty(),
            billing_rested: 0,
            billing_time: 0,
            expansion: Expansion::WrathOfTheLichKing,
        }
        .tokio_write_encrypted_server(&mut stream, &mut encryption)
        .await
        .unwrap();

        #[derive(Debug)]
        enum State {
            Login,
            LoggedIn(CharacterId),
        }

        let mut state = State::Login;

        let (reader, writer) = stream.into_split();

        let reader = Arc::new(Mutex::new((decryption, reader)));
        let writer = Arc::new(Mutex::new((encryption, writer)));

        loop {
            let opcode = {
                let (decrypter, reader) = &mut *reader.lock().await;
                match ClientOpcodeMessage::tokio_read_encrypted(reader, decrypter).await {
                    Ok(opcode) => opcode,
                    Err(ExpectedOpcodeError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                        tracing::trace!("client disconnected");
                        if let State::LoggedIn(character_id) = state {
                            self.world.take_session(character_id).await;
                        }
                        break;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        if let State::LoggedIn(character_id) = state {
                            self.world.take_session(character_id).await;
                        }
                        break;
                    }
                }
            };

            match (&state, opcode) {
                // loggging in
                (_, ClientOpcodeMessage::CMSG_PING(c)) => {
                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_PONG { sequence_id: c.sequence_id }
                        .tokio_write_encrypted_server(writer, encrypter)
                        .await
                        .unwrap();
                }
                (_, ClientOpcodeMessage::CMSG_CHAR_ENUM) => {
                    let characters = self
                        .characters
                        .get_by_account(account.id)
                        .await
                        .unwrap()
                        .into_iter()
                        .map(|c| wow_world_messages::wrath::Character {
                            guid: c.id.into(),
                            name: c.name,
                            race: c.race,
                            class: c.class,
                            gender: c.gender,
                            skin: c.skin_color,
                            face: c.face,
                            hair_style: c.hair_style,
                            hair_color: c.hair_color,
                            facial_hair: c.facial_style,
                            level: c.level,
                            area: c.zone,
                            map: c.map,
                            position: Vector3d {
                                x: c.position_x,
                                y: c.position_y,
                                z: c.position_z,
                            },
                            guild_id: 0,
                            flags: Default::default(),
                            recustomization_flags: 0,
                            first_login: false,
                            pet_display_id: 0,
                            pet_level: Level::zero(),
                            pet_family: CreatureFamily::None,
                            equipment: [Default::default(); 23],
                        })
                        .collect();

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_CHAR_ENUM { characters }.tokio_write_encrypted_server(writer, encrypter).await.unwrap();
                }
                (_, ClientOpcodeMessage::CMSG_CHAR_CREATE(c)) => {
                    let available = self.characters.name_available(&c.name).await.unwrap();

                    let response = match available {
                        true => {
                            let mut name_proper = c.name.to_ascii_lowercase();
                            if let Some(r) = name_proper.get_mut(0..1) {
                                r.make_ascii_uppercase();
                            };

                            match &c.name {
                                n if name_proper.ne(n) => WorldResult::CharNameFailure,
                                n if n.is_empty() => WorldResult::CharNameNoName,
                                n if n.len() < 2 => WorldResult::CharNameTooShort,
                                _ => {
                                    self.characters
                                        .create_character(account.id, CharacterCreate {
                                            name: c.name,
                                            race: c.race,
                                            class: c.class,
                                            gender: c.gender,
                                            skin_color: c.skin_color,
                                            face: c.face,
                                            hair_style: c.hair_style,
                                            hair_color: c.hair_color,
                                            facial_style: c.facial_hair,
                                            map: 0,               //
                                            zone: 1,              //
                                            position_x: -6240.32, // dwarf start zone
                                            position_y: 331.033,  //
                                            position_z: 382.758,  //
                                        })
                                        .await
                                        .map(|_| WorldResult::CharCreateSuccess)
                                        .unwrap_or_else(|e| {
                                            tracing::error!("unable to create character: {e}");
                                            WorldResult::AuthSystemError
                                        })
                                }
                            }
                        }
                        false => WorldResult::CharCreateNameInUse,
                    };

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_CHAR_CREATE { result: response }
                        .tokio_write_encrypted_server(writer, encrypter)
                        .await
                        .unwrap();
                }
                (_, ClientOpcodeMessage::CMSG_PLAYER_LOGIN(login)) => {
                    let character = self.characters.get(login.guid.try_into().unwrap()).await.unwrap().unwrap();
                    let friends = self.characters.get_friends(login.guid.try_into().unwrap()).await.unwrap();

                    {
                        let (encrypter, writer) = &mut *writer.lock().await;

                        SMSG_BINDPOINTUPDATE {
                            position: Vector3d {
                                x: character.position_x,
                                y: character.position_y,
                                z: character.position_z,
                            },
                            map: character.map,
                            area: character.zone,
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        let mask = RelationType::empty().set_friend().set_ignored().set_muted().set_recruitafriend();
                        SMSG_CONTACT_LIST {
                            list_mask: mask,
                            relations: friends
                                .into_iter()
                                .map(|(friend, flags, note)| Relation {
                                    guid: friend.into(),
                                    note,
                                    relation_mask: Relation_RelationType::new(flags, None),
                                })
                                .collect(),
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();
                        SMSG_ACTION_BUTTONS::Initial {
                            data: [ActionButton::default(); 144],
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        SMSG_INIT_WORLD_STATES {
                            area: character.zone,
                            map: character.map,
                            sub_area: Area::Teldrassil, // TODO: sub areas
                            states: vec![
                                WorldState {
                                    state: 3191, //Arena season
                                    value: 1,
                                },
                                WorldState {
                                    state: 3901, //Arena season progress
                                    value: 1,
                                },
                            ],
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();
                        MSG_SET_DUNGEON_DIFFICULTY_Server {
                            difficulty: DungeonDifficulty::Normal,
                            unknown1: 1,
                            is_in_group: false,
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        SMSG_LOGIN_SETTIMESPEED {
                            datetime: wow_world_messages::DateTime::new(23, wow_world_messages::Month::July, 15, wow_world_messages::Weekday::Saturday, 12, 12),
                            timescale: 0.01667f32,
                            unknown1: 0,
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        let factions = (0..128).map(|_| FactionInitializer::default()).collect();
                        SMSG_INITIALIZE_FACTIONS { factions }
                            .tokio_write_encrypted_server(&mut *writer, encrypter)
                            .await
                            .unwrap();

                        SMSG_FEATURE_SYSTEM_STATUS {
                            complaint_status: ComplaintStatus::EnabledWithAutoIgnore,
                            voice_chat_enabled: false,
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        SMSG_LOGIN_VERIFY_WORLD {
                            map: character.map,
                            position: Vector3d {
                                x: character.position_x,
                                y: character.position_y,
                                z: character.position_z,
                            },
                            orientation: 0.0, // todo
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        SMSG_TUTORIAL_FLAGS {
                            tutorial_data: [0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF],
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        {
                            let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

                            let data = self.characters.character_account_data(character.id).await.unwrap();

                            let mask = CacheMask::PER_CHARACTER_CACHE;
                            let mut cache_mask = CacheMask::default();
                            let cache_data = cache_mask.data_mut();
                            let mut idx = 0;

                            for (dtype, row) in data {
                                if mask & (1 << dtype) > 0 {
                                    *cache_data.get_mut(idx).unwrap() = Some(row.time);
                                    idx += 1;
                                }
                            }

                            SMSG_ACCOUNT_DATA_TIMES {
                                mask: cache_mask,
                                unix_time,
                                unknown1: 1,
                            }
                            .tokio_write_encrypted_server(&mut *writer, encrypter)
                            .await
                            .unwrap();
                        }

                        // this NEEDS to be set before we send packets
                        {
                            let guid = login.guid;
                            let update_mask = UpdatePlayer::builder()
                                .set_object_guid(guid)
                                .set_unit_bytes_0(character.race, character.class, character.gender, Power::Mana)
                                .set_unit_base_mana(100)
                                .set_object_scale_x(1.0)
                                .set_unit_health(100)
                                .set_unit_maxhealth(100)
                                .set_unit_level(character.level.as_int() as i32)
                                .set_unit_factiontemplate(1)
                                .set_unit_displayid(50)
                                .set_unit_nativedisplayid(50)
                                .finalize();

                            let update_flag = MovementBlock_UpdateFlag::empty()
                                .set_living(MovementBlock_UpdateFlag_Living::Living {
                                    backwards_running_speed: 4.5,
                                    backwards_swimming_speed: 0.0,
                                    fall_time: 0.0,
                                    flags: MovementBlock_MovementFlags::empty(),
                                    flight_speed: 0.0,
                                    backwards_flight_speed: 0.0,
                                    orientation: 0.0,
                                    position: Vector3d {
                                        x: character.position_x,
                                        y: character.position_y,
                                        z: character.position_z,
                                    },
                                    pitch_rate: 0.0,
                                    running_speed: 7.0,
                                    swimming_speed: 0.0,
                                    timestamp: 0,
                                    turn_rate: std::f32::consts::PI,
                                    walking_speed: 1.0,
                                })
                                .set_high_guid(wow_world_messages::wrath::MovementBlock_UpdateFlag_HighGuid {
                                    unknown0: if guid == login.guid { 0x2F } else { 0x08 },
                                })
                                .set_self();

                            SMSG_UPDATE_OBJECT {
                                objects: vec![Object::CreateObject2 {
                                    guid3: guid,
                                    mask2: UpdateMask::Player(update_mask),
                                    movement2: MovementBlock { update_flag },
                                    object_type: ObjectType::Player,
                                }],
                            }
                            .tokio_write_encrypted_server(writer, encrypter)
                            .await
                            .unwrap();
                        }
                    }

                    {
                        let (encrypter, writer) = &mut *writer.lock().await;
                        SMSG_TIME_SYNC_REQ { time_sync: 0 }
                            .tokio_write_encrypted_server(writer, encrypter)
                            .await
                            .unwrap();
                    }

                    self.world.create_session(character, writer.clone()).await.unwrap();

                    let cid: CharacterId = login.guid.try_into().unwrap();
                    tracing::trace!(guid = login.guid.guid(), character = cid.0, "logged in");
                    state = State::LoggedIn(cid);
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_LOGOUT_REQUEST) => {
                    {
                        let (encrypter, writer) = &mut *writer.lock().await;
                        SMSG_LOGOUT_RESPONSE {
                            result: LogoutResult::Success,
                            speed: LogoutSpeed::Instant,
                        }
                        .tokio_write_encrypted_server(&mut *writer, encrypter)
                        .await
                        .unwrap();

                        SMSG_LOGOUT_COMPLETE {}.tokio_write_encrypted_server(writer, encrypter).await.unwrap();
                    }

                    if let State::LoggedIn(character_id) = state {
                        self.world.take_session(character_id).await;
                    }

                    state = State::Login;
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_GMTICKET_GETTICKET) => {
                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_GMTICKET_GETTICKET::Default.tokio_write_encrypted_server(writer, encrypter).await.unwrap();
                }
                (_, ClientOpcodeMessage::CMSG_READY_FOR_ACCOUNT_DATA_TIMES) => {
                    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

                    let data = self.characters.account_data(account.id).await.unwrap();

                    let mask = CacheMask::GLOBAL_CACHE;
                    let mut cache_mask = CacheMask::default();
                    let cache_data = cache_mask.data_mut();
                    let mut idx = 0;

                    for (dtype, row) in data {
                        if mask & (1 << dtype) > 0 {
                            *cache_data.get_mut(idx).unwrap() = Some(row.time);
                            idx += 1;
                        }
                    }

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_ACCOUNT_DATA_TIMES {
                        mask: cache_mask,
                        unix_time,
                        unknown1: 1,
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap();
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_PLAYED_TIME(pt)) => {
                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_PLAYED_TIME {
                        show_on_ui: pt.show_on_ui,
                        total_played_time: 0, // TODO
                        level_played_time: 0, // TODO
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap()
                }
                (_, ClientOpcodeMessage::CMSG_NAME_QUERY(nq)) => {
                    if let Some(character) = self.characters.get(nq.guid.try_into().unwrap()).await.unwrap() {
                        let (encrypter, writer) = &mut *writer.lock().await;

                        SMSG_NAME_QUERY_RESPONSE {
                            guid: nq.guid,
                            character_name: character.name,
                            class: character.class,
                            gender: character.gender,
                            has_declined_names: wow_world_messages::wrath::SMSG_NAME_QUERY_RESPONSE_DeclinedNames::No,
                            race: character.race,
                            realm_name: "Realm".to_string(), // todo
                        }
                        .tokio_write_encrypted_server(writer, encrypter)
                        .await
                        .unwrap();
                    }
                }
                (_, ClientOpcodeMessage::CMSG_QUERY_TIME) => {
                    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

                    // seconds since mignight
                    let seconds_since_mignight = unix_time % 86400;
                    let seconds_to_next_mignight = 86400 - seconds_since_mignight;

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_QUERY_TIME_RESPONSE {
                        time: unix_time,
                        time_until_daily_quest_reset: seconds_to_next_mignight,
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap();
                }
                (_, ClientOpcodeMessage::CMSG_WORLD_STATE_UI_TIMER_UPDATE) => {
                    let unix_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_WORLD_STATE_UI_TIMER_UPDATE { time: unix_time }
                        .tokio_write_encrypted_server(writer, encrypter)
                        .await
                        .unwrap();
                }
                (State::LoggedIn(guid), ClientOpcodeMessage::CMSG_MESSAGECHAT(chat)) => {
                    let chat_type = match &chat.chat_type {
                        wow_world_messages::wrath::CMSG_MESSAGECHAT_ChatType::Say => Some(SMSG_MESSAGECHAT_ChatType::Say { target6: (*guid).into() }),
                        wow_world_messages::wrath::CMSG_MESSAGECHAT_ChatType::Afk => Some(SMSG_MESSAGECHAT_ChatType::Afk { target6: (*guid).into() }),
                        _ => {
                            tracing::trace!("unhandled message {:?}", chat);
                            None
                        }
                    };

                    if let Some(chat_type) = chat_type {
                        let (encrypter, writer) = &mut *writer.lock().await;
                        SMSG_MESSAGECHAT {
                            chat_type,
                            language: chat.language,
                            message: chat.message,
                            sender: (*guid).into(),
                            ..Default::default()
                        }
                        .tokio_write_encrypted_server(writer, encrypter)
                        .await
                        .unwrap();
                    }
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_SET_ACTIONBAR_TOGGLES(_)) => {
                    // TODO: set toggles on character
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_TIME_SYNC_RESP(_)) => {
                    // TODO: set time sync on character
                }
                (State::LoggedIn(guid), ClientOpcodeMessage::CMSG_ZONEUPDATE(_)) => {
                    let character = self.characters.get((*guid).try_into().unwrap()).await.unwrap().unwrap();

                    let (encrypter, writer) = &mut *writer.lock().await;

                    // TODO: set zone on character
                    SMSG_INIT_WORLD_STATES {
                        area: character.zone,
                        map: character.map,
                        sub_area: Area::Teldrassil, // TODO: sub areas
                        states: vec![
                            WorldState {
                                state: 3191, //Arena season
                                value: 1,
                            },
                            WorldState {
                                state: 3901, //Arena season progress
                                value: 1,
                            },
                        ],
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap();
                }
                (State::LoggedIn(character_id), ClientOpcodeMessage::CMSG_SET_ACTIVE_MOVER(mover)) => {
                    // TODO: we probably want to do something with this
                    if Guid::from(character_id) != mover.guid {
                        warn!(active = mover.guid.guid(), player = character_id.0, "received mover for non-player");
                    }
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_BATTLEFIELD_STATUS) => {
                    // TODO: integrate battlegrounds

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_BATTLEFIELD_STATUS {
                        status_id: SMSG_BATTLEFIELD_STATUS_StatusId::None,
                        arena_type: ArenaType::NotArena,
                        battleground_type: BattlegroundType::None,
                        client_instance_id: 0,
                        is_arena: 0,
                        rated: false,
                        unknown1: 0x1F90,
                        maximum_level: 0,
                        minimum_level: 0,
                        queue_slot: 0,
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap();
                }
                (State::LoggedIn(_), ClientOpcodeMessage::CMSG_STANDSTATECHANGE(_)) => {
                    // TODO: get sit / stand in bevy
                }
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_FORWARD(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_BACKWARD(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_STOP(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_STRAFE_LEFT(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_STRAFE_RIGHT(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_STOP_STRAFE(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_JUMP(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_TURN_LEFT(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_TURN_RIGHT(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_PITCH_UP(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_PITCH_DOWN(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_STOP_PITCH(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_SET_RUN_MODE(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_SET_WALK_MODE(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_FALL_LAND(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_START_SWIM(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_STOP_SWIM(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_STOP_TURN(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_SET_FACING(box data)) => self.handle_movement(guid, data.clone()).await,
                (State::LoggedIn(guid), ClientOpcodeMessage::MSG_MOVE_HEARTBEAT(box data)) => self.handle_movement(guid, data.clone()).await,
                (
                    _,
                    ClientOpcodeMessage::CMSG_CHANNEL_VOICE_ON
                    | ClientOpcodeMessage::CMSG_VOICE_SESSION_ENABLE(_)
                    | ClientOpcodeMessage::CMSG_SET_ACTIVE_VOICE_CHANNEL(_),
                ) => {
                    // voice is not supported
                }
                (State::LoggedIn(guid), ClientOpcodeMessage::CMSG_CONTACT_LIST(_)) => {
                    let friends = self.characters.get_friends((*guid).try_into().unwrap()).await.unwrap();
                    let mask = RelationType::empty().set_friend().set_ignored().set_muted().set_recruitafriend();

                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_CONTACT_LIST {
                        list_mask: mask,
                        relations: friends
                            .into_iter()
                            .map(|(friend, flags, note)| Relation {
                                guid: friend.into(),
                                note,
                                relation_mask: Relation_RelationType::new(flags, None),
                            })
                            .collect(),
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap();
                }
                (State::LoggedIn(character_id), ClientOpcodeMessage::CMSG_ADD_FRIEND(friend)) => {
                    let response = match self
                        .characters
                        .add_friend_by_name(*character_id, &friend.name, Some(&friend.note))
                        .await
                        .unwrap()
                    {
                        Some(friend) => SMSG_FRIEND_STATUS {
                            guid: friend.into(),
                            result: wow_world_messages::wrath::FriendResult::AddedOffline,
                        },
                        None => SMSG_FRIEND_STATUS {
                            guid: Guid::zero(),
                            result: wow_world_messages::wrath::FriendResult::NotFound,
                        },
                    };

                    let (encrypter, writer) = &mut *writer.lock().await;

                    response.tokio_write_encrypted_server(writer, encrypter).await.unwrap();
                }
                (_, ClientOpcodeMessage::CMSG_REALM_SPLIT(rs)) => {
                    let (encrypter, writer) = &mut *writer.lock().await;
                    SMSG_REALM_SPLIT {
                        realm_id: rs.realm_id,
                        state: RealmSplitState::Normal,
                        split_date: "01/01/01".into(),
                    }
                    .tokio_write_encrypted_server(writer, encrypter)
                    .await
                    .unwrap()
                }
                (state, e) => {
                    tracing::trace!(state = match state {
                        State::Login => "login",
                        State::LoggedIn(_) => "logged in",
                    }, opcode = ?e);
                }
            }
        }

        Ok(())
    }

    async fn handle_movement(&self, guid: &CharacterId, data: impl MovementMessage) {
        let update = data.get_movement_info();
        self.world.movement_update(*guid, update);
    }

    /// Start the world server, running the various tasks that it is comprised of
    pub async fn start(self, cancel: CancellationToken) -> Result<()> {
        let server = Arc::new(self);

        try_join!(
            flatten(tokio::task::Builder::new().name("world::heartbeat").spawn({
                let cloned = server.clone();
                async move { cloned.auth_server_heartbeat().await.context("heartbeat error") }
            })?),
            flatten(tokio::task::Builder::new().name("world::clients").spawn({
                let cloned = server.clone();
                async move { cloned.accept_clients().await.context("client error") }
            })?),
            flatten(tokio::task::Builder::new().name("world::packets").spawn({
                let cloned = server.clone();
                async move {
                    cloned.world.packet_handler().await;
                    Ok(())
                }
            })?),
            async {
                cancel.cancelled().await;
                Result::<()>::Err(anyhow::anyhow!("cancelled"))
            }
        )?;

        Ok(())
    }
}

// just ripped this from wrath-rs for now. lets replace it once we have it working
pub trait MovementMessage: Sync + ServerMessage + ClientMessage {
    #[allow(unused)]
    fn get_guid(&self) -> Guid;
    fn get_movement_info(&self) -> MovementInfo;
}

macro_rules! define_movement_packet {
    ($packet_type:ty) => {
        impl MovementMessage for $packet_type {
            fn get_guid(&self) -> Guid {
                self.guid
            }

            fn get_movement_info(&self) -> MovementInfo {
                self.info.clone()
            }
        }
    };
}

define_movement_packet!(MSG_MOVE_START_FORWARD);
define_movement_packet!(MSG_MOVE_START_BACKWARD);
define_movement_packet!(MSG_MOVE_STOP);
define_movement_packet!(MSG_MOVE_STOP_TURN);
define_movement_packet!(MSG_MOVE_START_STRAFE_LEFT);
define_movement_packet!(MSG_MOVE_START_STRAFE_RIGHT);
define_movement_packet!(MSG_MOVE_STOP_STRAFE);
define_movement_packet!(MSG_MOVE_JUMP);
define_movement_packet!(MSG_MOVE_START_TURN_LEFT);
define_movement_packet!(MSG_MOVE_START_TURN_RIGHT);
define_movement_packet!(MSG_MOVE_START_PITCH_UP);
define_movement_packet!(MSG_MOVE_START_PITCH_DOWN);
define_movement_packet!(MSG_MOVE_STOP_PITCH);
define_movement_packet!(MSG_MOVE_SET_RUN_MODE);
define_movement_packet!(MSG_MOVE_SET_WALK_MODE);
define_movement_packet!(MSG_MOVE_FALL_LAND);
define_movement_packet!(MSG_MOVE_START_SWIM);
define_movement_packet!(MSG_MOVE_STOP_SWIM);
define_movement_packet!(MSG_MOVE_SET_FACING);
define_movement_packet!(MSG_MOVE_HEARTBEAT);
