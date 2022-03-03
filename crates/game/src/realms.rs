//! realms
//!
//! The realms module handles everything regarding managing
//! realm and realmlists.

use std::time::SystemTime;

use async_trait::async_trait;
use derive_more::{From, Into};
use enumflags2::bitflags;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use strum_macros::ToString;
use thiserror::Error;

/// The various flags that a realm can have.
/// They are implemented as BitFlags.
#[repr(u8)]
#[bitflags]
#[derive(Clone, Copy, Debug)]
pub enum RealmFlags {
    Invalid = 0b0000_0001,
    Offline = 0b0000_0010,
    SpecifyBuild = 0b0000_0100,
    Unknown1 = 0b0000_1000,
    Unknown2 = 0b0001_0000,
    Recommended = 0b0010_0000,
    New = 0b0100_0000,
    Full = 0b1000_0000,
}

/// The various types of realm.
/// For more, see <https://wow.tools/dbc/?dbc=cfg_configs&build=3.3.5.12340>
#[repr(u8)]
#[derive(Clone, Copy, Type, Debug, IntoPrimitive, ToString)]
pub enum RealmType {
    Normal = 0,
    PVP = 1,
    RP = 6,
    RPPvP = 8,
}

/// A marker for a realm id.
#[derive(Type, Clone, Debug, From, Into, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[sqlx(transparent)]
pub struct RealmId(pub u32);

/// The basic realm object.
#[derive(Clone, Debug)]
pub struct Realm {
    pub id: RealmId,
    pub name: String,
    pub realm_type: RealmType,
    pub build: u32,
    pub external_address: String,
    pub local_address: String,
    pub local_subnet_mask: String,
    pub port: u16,
    pub flags: u8, // BitFlags<RealmFlags>
    pub timezone: u8,
    pub population: f32,
}

/// A trait that models a realmlist.
#[async_trait]
pub trait RealmList: Send + Sync {
    /// Return the list of realms sorted by id.
    async fn realms(&self) -> Vec<Realm>;

    async fn update_status(&self, online: Vec<(u8, RealmFlags)>) -> Result<(), RealmListError>;

    /// Update the uptime counter for a server that started
    /// at the given `start` time.
    async fn set_uptime(
        &self,
        id: RealmId,
        start: SystemTime,
        population: u32,
    ) -> Result<(), RealmListError>;
}

/// Errors that may occur when running realmlist operations.
#[derive(Error, Debug)]
pub enum RealmListError {
    #[error("start time is in the future")]
    StartTimeInFuture,
    #[error("start time is too large to be stored")]
    StartTimeTooLarge,
    #[error("error in persistence layer: {0}")]
    PersistError(String),
}
