//! world
//!
//! This module defines a bevy ECS world that is used to drive the game.

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Result, anyhow};
use azerust_game::{
    Guid as GameGuid,
    accounts::AccountService,
    characters::{Character, CharacterId, CharacterService},
    realms::{RealmId, RealmList},
};
use bevy::{app::ScheduleRunnerPlugin, prelude::*, utils::HashSet};
use time::OffsetDateTime;
use tokio::{
    io::AsyncWriteExt,
    sync::{
        Mutex, RwLock,
        mpsc::{UnboundedReceiver, UnboundedSender},
    },
    task::JoinHandle,
};
use wow_srp::wrath_header::ServerEncrypterHalf;
use wow_world_messages::wrath::{
    MovementBlock, MovementBlock_UpdateFlag, MovementInfo, MovementInfo_MovementFlags, Object, ObjectType, Power, SMSG_UPDATE_OBJECT, UpdateMask, UpdatePlayer,
    Vector3d, opcodes::ServerOpcodeMessage,
};

#[allow(unused)]
pub struct World<A: AccountService, R: RealmList, C: CharacterService, W: AsyncWriteExt> {
    id: RealmId,
    accounts: A,
    realms: R,
    characters: C,
    sessions: Arc<RwLock<HashMap<CharacterId, Arc<Mutex<(ServerEncrypterHalf, W)>>>>>,
    start: OffsetDateTime,
    ecs_handle: JoinHandle<()>,
    connection_updates: UnboundedSender<ConnectionEvent>,
    movement_updates: UnboundedSender<MovementUpdate>,
    send_packets: UnboundedSender<(CharacterId, ServerOpcodeMessage)>,
    dump_packets: Mutex<Option<UnboundedReceiver<(CharacterId, ServerOpcodeMessage)>>>,
}

#[derive(Resource)]
struct ReceiverResource<T>(UnboundedReceiver<T>);

impl<A: AccountService, R: RealmList, C: CharacterService, W: AsyncWriteExt + Send + Unpin> World<A, R, C, W> {
    pub fn new(id: RealmId, accounts: A, realms: R, characters: C) -> Self {
        let (s_1, r_1) = tokio::sync::mpsc::unbounded_channel();
        let (s_2, r_2) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::task::Builder::new().name("world::bevy").spawn_blocking(|| run_world(r_1, r_2)).unwrap();
        let (send_packets, dump_packets) = tokio::sync::mpsc::unbounded_channel();

        Self {
            id,
            accounts,
            realms,
            characters,
            sessions: Default::default(),
            start: OffsetDateTime::now_utc(),
            ecs_handle: handle,
            connection_updates: s_1,
            movement_updates: s_2,
            send_packets,
            dump_packets: Mutex::new(Some(dump_packets)),
        }
    }

    pub async fn packet_handler(&self) {
        let mut dump_packets = {
            let mut dump_packets = self.dump_packets.lock().await;
            dump_packets.take().unwrap()
        };

        while let Some((character_id, packet)) = dump_packets.recv().await {
            let sessions = self.sessions.read().await;
            if let Some(writer) = sessions.get(&character_id) {
                let (encrypter, writer) = &mut *writer.lock().await;
                packet.tokio_write_encrypted_server(writer, encrypter).await.unwrap();
            }
        }
    }

    pub async fn create_session(
        &self,
        character: Character,
        writer: Arc<Mutex<(ServerEncrypterHalf, W)>>,
    ) -> Result<(), (anyhow::Error, Arc<Mutex<(ServerEncrypterHalf, W)>>)> {
        {
            let mut sessions = self.sessions.write().await;
            if sessions.contains_key(&character.id) {
                return Err((anyhow!("account already logged in"), writer));
            }

            sessions.insert(character.id, writer);
        }

        tracing::info!("inserting connection");
        self.connection_updates
            .send(ConnectionEvent::Connection(
                Connection {
                    id: character.id,
                    class: character.class,
                    face: character.face,
                    facial_style: character.facial_style,
                    gender: character.gender,
                    hair_color: character.hair_color,
                    hair_style: character.hair_style,
                    map: character.map.as_int(),
                    name: character.name,
                    orientation: 0.0f32,
                    race: character.race,
                    skin_color: character.skin_color,
                    x: character.position_x,
                    y: character.position_y,
                    z: character.position_z,
                    zone: character.zone.as_int(),
                },
                PlayerBroadcast(character.id, self.send_packets.clone()),
            ))
            .unwrap();

        Ok(())
    }

    pub(crate) fn movement_update(&self, guid: CharacterId, data: MovementInfo) {
        self.movement_updates.send(MovementUpdate(guid, data)).unwrap();
    }

    pub(crate) async fn take_session(&self, id: CharacterId) {
        tracing::info!("removing connection");
        self.sessions.write().await.remove(&id);
        self.connection_updates.send(ConnectionEvent::Disconnection(id)).unwrap();
    }
}

fn run_world(receiver: UnboundedReceiver<ConnectionEvent>, receiver2: UnboundedReceiver<MovementUpdate>) {
    trace!("running world");
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))))
        .add_systems(FixedUpdate, update_position)
        .add_event::<ConnectionEvent>()
        .add_event::<MovementUpdate>()
        .add_systems(FixedUpdate, movement_update)
        .add_systems(FixedPreUpdate, drain_connection_updates)
        .add_systems(FixedPreUpdate, drain_movement_updates)
        .insert_resource(Time::<Fixed>::from_hz(10.0))
        .insert_resource(CharacterEntities(Default::default()))
        .insert_resource(ReceiverResource(receiver))
        .insert_resource(ReceiverResource(receiver2))
        .run();
    trace!("closing world");
}

fn drain_connection_updates(mut queue: ResMut<ReceiverResource<ConnectionEvent>>, mut events: EventWriter<ConnectionEvent>) {
    while let Ok(event) = queue.0.try_recv() {
        events.send(event);
    }
}

fn drain_movement_updates(mut queue: ResMut<ReceiverResource<MovementUpdate>>, mut events: EventWriter<MovementUpdate>) {
    while let Ok(event) = queue.0.try_recv() {
        events.send(event);
    }
}

pub struct Connection {
    pub id: CharacterId,
    pub name: String,
    pub race: wow_world_messages::wrath::Race,
    pub class: wow_world_messages::wrath::Class,
    pub gender: wow_world_messages::wrath::Gender,
    pub skin_color: u8,
    pub face: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub facial_style: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: f32,
    pub map: u32,
    pub zone: u32,
}

#[derive(Event)]
pub enum ConnectionEvent {
    Connection(Connection, PlayerBroadcast),
    Disconnection(CharacterId),
}

#[derive(Event)]
pub struct MovementUpdate(pub CharacterId, pub MovementInfo);

#[derive(Component, Eq, PartialEq, Hash)]
struct GameObject(GameGuid, UpdatePlayer);

impl Default for GameObject {
    fn default() -> Self {
        panic!("entities must have a guid")
    }
}

impl GameObject {
    pub fn update_mask(&self) -> UpdateMask {
        UpdateMask::Player(self.1.clone())
    }
}

#[derive(Resource)]
struct CharacterEntities(HashMap<CharacterId, Entity>);

type ServerMessageEvent = (CharacterId, ServerOpcodeMessage);

#[derive(Clone)]
pub struct PlayerBroadcast(pub CharacterId, pub UnboundedSender<ServerMessageEvent>);

impl PlayerBroadcast {
    pub fn send(&self, message: ServerOpcodeMessage) {
        if let Err(e) = self.1.send((self.0, message)) {
            tracing::error!("failed to send message: {e}");
        }
    }
}

#[derive(Component)]
#[require(Name, Position, GameObject)]
#[allow(unused)]
struct Player {
    broadcast: PlayerBroadcast,
    // this is infrequently updated
    tracked_guids: HashSet<GameGuid>,
    race: wow_world_messages::wrath::Race,
    class: wow_world_messages::wrath::Class,
    gender: wow_world_messages::wrath::Gender,
    skin_color: u8,
    face: u8,
    hair_style: u8,
    hair_color: u8,
    facial_style: u8,
}

#[derive(Component, Default)]
#[allow(unused)]
struct Name(String);

#[derive(Component)]
#[allow(unused)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
    orientation: f32,
    #[allow(unused)]
    map: u32,
    #[allow(unused)]
    zone: u32,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: -8949.94f32,
            y: -132.50f32,
            z: 83.53f32,
            orientation: 0.0,
            map: 0,
            zone: 1,
        }
    }
}

/// Add and remove players from the world
#[tracing::instrument(skip_all)]
fn update_position(
    mut commands: Commands,
    mut events: EventReader<ConnectionEvent>,
    mut movement: EventWriter<MovementUpdate>,
    mut players: ResMut<CharacterEntities>,
) {
    for event in events.read() {
        tracing::info!("connection event");
        match &event {
            ConnectionEvent::Connection(conn, broadcast) => {
                info!(id = conn.id.0, name = conn.name, "adding player");
                let _player = commands.spawn((
                    Player {
                        broadcast: broadcast.to_owned(),
                        class: conn.class,
                        race: conn.race,
                        gender: conn.gender,
                        skin_color: conn.skin_color,
                        face: conn.face,
                        hair_style: conn.hair_style,
                        hair_color: conn.hair_color,
                        facial_style: conn.facial_style,
                        tracked_guids: Default::default(),
                    },
                    Name(conn.name.to_owned()),
                    Position {
                        map: conn.map,
                        zone: conn.zone,
                        orientation: conn.orientation,
                        x: conn.x,
                        y: conn.y,
                        z: conn.z,
                    },
                    GameObject(
                        conn.id.into(),
                        UpdatePlayer::builder()
                            .set_object_guid(conn.id.into())
                            .set_unit_bytes_0(conn.race, conn.class, conn.gender, Power::Mana)
                            .set_object_scale_x(1.0)
                            .set_unit_health(100)
                            .set_unit_base_mana(100)
                            .set_unit_maxhealth(100)
                            .set_unit_level(1)
                            .set_unit_factiontemplate(1)
                            .set_unit_displayid(50)
                            .set_unit_nativedisplayid(50)
                            .finalize(),
                    ),
                ));

                movement.send(MovementUpdate(conn.id, MovementInfo {
                    fall_time: 0.0,
                    orientation: conn.orientation,
                    position: Vector3d {
                        x: conn.x,
                        y: conn.y,
                        z: conn.z,
                    },
                    flags: MovementInfo_MovementFlags::empty(),
                    timestamp: 0, // why??
                }));
            }
            ConnectionEvent::Disconnection(id) => {
                let character_entity = players.0.remove(id);
                info!(id = id.0, "removing player");
                if let Some(character_entity) = character_entity {
                    commands.entity(character_entity).despawn();
                }
            }
        }
    }
}

fn movement_update(mut events: EventReader<MovementUpdate>, mut players: Query<(&mut Position, &mut Player, &GameObject)>) {
    let events: Vec<_> = events.read().collect();

    for (mut pos, mut player, gameobject) in players.iter_mut() {
        let mut objects = Vec::new();

        for event in events.iter() {
            // TODO: limit to players in range
            let moved_guid = event.0.into();
            let player_guid = gameobject.0;

            let creating_self = player_guid == moved_guid;

            if creating_self {
                pos.x = event.1.position.x;
                pos.y = event.1.position.y;
                pos.z = event.1.position.z;
                pos.orientation = event.1.orientation;
            }

            let new_guid = player.tracked_guids.insert(moved_guid);

            if new_guid {
                tracing::debug!(
                    moved = moved_guid.guid(),
                    player = player_guid.guid(),
                    self = creating_self,
                    "tracking for first time"
                );
            }

            let movement_info = event.1.clone();

            let mut update_flag = MovementBlock_UpdateFlag::empty()
                .set_living(movement_info.to_movement_block_update_flag_living(
                    0.0,                  /* backwards_flight_speed */
                    4.5,                  /* backwards_running_speed */
                    0.0,                  /* backwards_swimming_speed */
                    0.0,                  /* flight_speed */
                    0.0,                  /* pitch_rate */
                    7.0,                  /* running_speed */
                    0.0,                  /* swimming_speed */
                    std::f32::consts::PI, /* turn_rate */
                    1.0,                  /* walking_speed */
                    None,                 /* spline_enabled */
                                          // 4.5,                  // MOVE_FLIGHT_BACK
                                          // 4.5,                  // MOVE_RUN_BACK
                                          // 2.5,                  // MOVE_SWIM_BACK
                                          // 7.0,                  // MOVE_FLIGHT
                                          // 3.14,                 // MOVE_PITCH_RATE
                                          // 7.0,                  // MOVE_RUN
                                          // 4.722222,             // MOVE_SWIM
                                          // std::f32::consts::PI, // MOVE_TURN_RATE
                                          // 2.5,                  // MOVE_WALK
                                          // None,                 /* spline_enabled */
                ))
                .set_high_guid(wow_world_messages::wrath::MovementBlock_UpdateFlag_HighGuid {
                    unknown0: if creating_self { 0x2F } else { 0x08 },
                });
            /*.set_LOW_GUID(wow_world_messages::wrath::MovementBlock_UpdateFlag_LowGuid {
            unknown1: object_guid.guid() as u32,
            })*/

            if creating_self {
                update_flag = update_flag.set_self()
            }

            //Copy the update mask and mark every field dirty, so that we send everything we need to know
            let mut all_dirty_update_mask = gameobject.update_mask();
            match all_dirty_update_mask {
                UpdateMask::Player(ref mut inner) => inner.mark_fully_dirty(),
                _ => unimplemented!(),
            }

            let update_type = if creating_self {
                Object::CreateObject2 {
                    guid3: moved_guid,
                    mask2: all_dirty_update_mask,
                    movement2: MovementBlock { update_flag },
                    object_type: ObjectType::Player,
                }
            } else {
                Object::CreateObject {
                    guid3: moved_guid,
                    mask2: all_dirty_update_mask,
                    movement2: MovementBlock { update_flag },
                    object_type: ObjectType::Player,
                }
            };

            objects.push(update_type);
        }
        player
            .broadcast
            .send(ServerOpcodeMessage::SMSG_UPDATE_OBJECT(Box::new(SMSG_UPDATE_OBJECT { objects })));
    }
}
