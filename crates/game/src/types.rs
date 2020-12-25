use num_enum::IntoPrimitive;
use std::convert::TryFrom;
use strum_macros::EnumString;

#[repr(u8)]
#[derive(EnumString, IntoPrimitive)]
pub enum Locale {
    enUS = 0,
    koKR,
    frFR,
    deDE,
    zhCN,
    esES,
    esMX,
    ruRU,
}
