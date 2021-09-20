use num_enum::IntoPrimitive;
use serde::Serialize;

#[cfg(feature = "auth")]
pub mod auth;

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
