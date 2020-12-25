use async_trait::async_trait;
use derive_more::{From, Into};
use enumflags2::BitFlags;
use num_enum::IntoPrimitive;
use sqlx::Type;

#[repr(u8)]
#[derive(Clone, Copy, BitFlags, Debug)]
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

#[repr(u8)]
#[derive(Clone, Copy, Type, Debug, IntoPrimitive)]
pub enum RealmType {
    Normal = 0,
    PVP = 1,
    RP = 6,
    RPPvP = 8,
}

#[derive(Type, Clone, Debug, From, Into, Copy)]
#[sqlx(transparent)]
pub struct RealmId(u32);

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

#[async_trait]
pub trait RealmList {
    /// Return the list of realms sorted by id.
    async fn realms(&self) -> Vec<Realm>;
}
