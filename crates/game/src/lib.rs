//! game
//!
//! This crate models much of the core gameplay systems of
//! World of Warcraft.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications,
    clippy::useless_conversion,
    clippy::unwrap_used,
    clippy::todo,
    clippy::unimplemented
)]

use serde::{Deserialize, Serialize};
use sqlx::Type;

pub mod accounts;
pub mod characters;
pub mod realms;
pub mod types;

#[derive(PartialEq, Debug, Eq, Clone, Copy, Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct WowId(u64);

#[derive(Debug, Clone, Copy)]
pub enum EntityType {
    Player = 0x0000,
    ItemOrContainer = 0x4000,
    GameObject = 0xF110,
    Transport = 0xF120,
    Unit = 0xF130,
    Pet = 0xF140,
    Vehicle = 0xF150,
    DynamicObject = 0xF100,
    Corpse = 0xF101,
    MoTransport = 0x1FC0,
    Group = 0x1F50,
    Instance = 0x1F42,
}

impl WowId {
    pub fn new(r#type: EntityType, low: u32, mid: u32) -> Self {
        Self((low as u64) | ((mid as u64) << 24) | ((r#type as u64) << 48))
    }
}
