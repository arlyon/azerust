//! types
//!
//! This module contains types that don't really fit elsewhere.

use num_enum::IntoPrimitive;
use strum_macros::EnumString;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(EnumString, IntoPrimitive, Copy, Clone, Debug)]
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
