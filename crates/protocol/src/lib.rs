use azerust_game::{realms::RealmId, WowId};
use num_enum::IntoPrimitive;
use serde::Serialize;

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

#[derive(Debug, Serialize, Clone, Copy)]
pub struct Item {
    pub display: u32,
    pub inventory: u8,
    pub aura: u32,
}
