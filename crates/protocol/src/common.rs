use num_enum::IntoPrimitive;
use serde::Serialize;

#[repr(u8)]
#[derive(Serialize, IntoPrimitive, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(into = "u8")]
pub enum ClientVersion {
    Classic = 0,
    BurningCrusade = 1,
    Wotlk = 2,
}
