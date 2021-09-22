use azerust_game::{
    characters::{AccountData, Character},
    realms::RealmId,
    WowId,
};
use num_enum::IntoPrimitive;
use serde::Serialize;
use world::ResponseCode;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "world")]
pub mod header_crypto;
#[cfg(feature = "world")]
pub mod world;

#[repr(u8)]
#[derive(Serialize, IntoPrimitive, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(into = "u8")]
pub enum ClientVersion {
    Classic = 0,
    BurningCrusade = 1,
    Wotlk = 2,
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

#[derive(Debug, Clone)]
pub struct Addon {
    pub name: String,
    signature: bool,
    pub crc: u32,
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
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Item {
    pub display: u32,
    pub inventory: u8,
    pub aura: u32,
}

#[derive(Debug)]
pub enum ServerPacket {
    AuthResponse,
    AddonInfo(Vec<Addon>),
    ClientCacheVersion(u32),
    TutorialData,
    Pong(u32),
    CharEnum(Vec<(Character, [Item; 23])>),
    AccountDataTimes(Box<AccountData>),
    RealmSplit { realm: u32 },
    CharacterCreate(ResponseCode),
    CharacterDelete(ResponseCode),
}
