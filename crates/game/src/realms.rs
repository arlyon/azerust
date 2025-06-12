//! realms
//!
//! The realms module handles everything regarding managing
//! realm and realmlists.

use std::net::IpAddr;

use derive_more::{From, Into};
use enumflags2::bitflags;
use num_enum::IntoPrimitive;
use serde::{Deserialize, Serialize};
use strum_macros::ToString;
use thiserror::Error;
use time::OffsetDateTime;

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
#[derive(Clone, Copy, Debug, IntoPrimitive, ToString)]
pub enum RealmType {
    Normal = 0,
    PVP = 1,
    RP = 6,
    RPPvP = 8,
}

/// A marker for a realm id.
#[derive(Clone, Debug, From, Into, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealmId(pub u32);

/// The basic realm object.
#[derive(Clone, Debug)]
pub struct Realm {
    pub id: RealmId,
    pub name: String,
    pub realm_type: RealmType,
    pub build: u32,
    pub external_address: IpAddr,
    pub local_address: IpAddr,
    pub local_subnet_mask: IpAddr,
    pub port: u16,
    pub flags: u8,
    pub timezone: u8,
    pub population: f32,
}

/// A trait that models a realmlist.
pub trait RealmList: Send + Sync {
    /// Return the list of realms sorted by id.
    fn realms(&self) -> impl Future<Output = Result<Vec<Realm>, RealmListError>> + Send;

    fn update_status(&self, online: Vec<(u8, RealmFlags)>) -> impl Future<Output = Result<(), RealmListError>> + Send;

    /// Update the uptime counter for a server that started
    /// at the given `start` time.
    fn set_uptime(&self, id: RealmId, start: OffsetDateTime, population: u16) -> impl Future<Output = Result<(), RealmListError>> + Send;

    /// Create a new realm in the realmlist.
    fn create_realm(
        &self,
        name: &str,
        address: &IpAddr,
        local_address: &IpAddr,
        local_subnet_mask: &IpAddr,
        port: u16,
    ) -> impl Future<Output = Result<Realm, RealmListError>> + Send;
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
