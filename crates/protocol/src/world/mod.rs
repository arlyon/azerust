use assert_size_attribute::assert_eq_size;
use azerust_game::{
    characters::{AccountData, AccountDataType, Character},
    realms::RealmId,
    WowId,
};
use serde::Serialize;

pub mod opcode;
pub use opcode::*;

#[derive(Debug, Clone)]
pub struct Addon {
    pub name: String,
    #[allow(dead_code)]
    signature: bool,
    pub crc: u32,
    #[allow(dead_code)]
    crc2: u32,
}

impl Addon {
    pub fn new(name: String, signature: bool, crc: u32, crc2: u32) -> Self {
        Self {
            name,
            signature,
            crc,
            crc2,
        }
    }
}

#[derive(Debug)]
pub struct AuthSession {
    pub build: u32,
    pub server_id: u32,
    pub username: String,
    pub login_server_type: u32,
    pub region_id: u32,
    pub local_challenge: [u8; 4],
    pub client_proof: [u8; 20],
    pub addons: Vec<Addon>,

    pub battlegroup_id: u32,
    pub realm_id: RealmId,
    pub dos_response: u64,
}

#[derive(Debug)]
pub enum ClientPacket {
    AuthSession(AuthSession),
    KeepAlive,
    Ping {
        seq: u32,
        latency: u32,
    },
    ReadyForAccountDataTimes,
    CharEnum,
    RealmSplit {
        realm: u32,
    },
    CharacterCreate {
        name: String,
        race: u8,
        class: u8,
        gender: u8,
        skin_color: u8,
        face: u8,
        hair_style: u8,
        hair_color: u8,
        facial_style: u8,
    },
    PlayerLogin(WowId),
    CharacterDelete(WowId),
    NameQuery(WowId),
    SetActiveVoiceChannel((u32, String)),
    UpdateAccountData((u32, u32, u32)),
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Item {
    pub display: u32,
    pub inventory: u8,
    pub aura: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum RelationType {
    None = 0x00,
    Friend = 0x01,
    Ignore = 0x02,
    Muted = 0x04,
    RecruitAFriend = 0x08,
}

#[derive(Debug)]
pub enum ServerPacket {
    AuthResponse,
    AuraUpdateAll,
    AddonInfo(Vec<Addon>),
    ClientCacheVersion(u32),
    ContactList(Vec<RelationType>),
    TutorialData,
    Pong(u32),
    CharEnum(Vec<(Character, [Item; 23])>),
    AccountDataTimes(Box<AccountData>, AccountDataType),
    RealmSplit {
        realm: u32,
    },
    CharacterCreate(ResponseCode),
    CharacterDelete(ResponseCode),
    VerifyWorld(VerifyWorld),
    FeatureSystemStatus,
    LearnedDanceMoves,
    Motd(String),
    SysMessage(String),
    CinematicStart(u32),
    SocialList,
    BindPointUpdate {
        x: f32,
        y: f32,
        z: f32,
        map: u32,
        area: u32,
    },
    TalentsInfo,
    InstanceDifficulty,
    InitialSpells,
    SendUnlearnSpells,
    InitialActionButtons,
    InitializeFactions,
    AllAchievementData,
    EquipmentList,
    LoginSetTimeSpeed,
    SetForcedReactions,
    SendTimeSync(u32),
    InitialWorldStates(Character),
}

#[repr(packed)]
#[assert_eq_size([u8; 4 + 4 * 4])]
#[derive(Serialize, Clone, Copy, Debug)]
pub struct VerifyWorld {
    pub map_id: u32,
    pub position: Position,
}

#[repr(packed)]
#[assert_eq_size([u8; 4 * 4])]
#[derive(Serialize, Clone, Copy, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: f32,
}
