use thiserror::Error;
pub use wow_world_messages::wrath::{Area, Class, Gender, Level, Map, Race};

use crate::{EntityType, Guid, accounts::AccountId, new_guid};

#[derive(Error, Debug, Clone, Copy)]
pub enum TryFromWowIdError {
    #[error("invalid entity {0}")]
    InvalidEntityType(u16),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct CharacterId(pub u32);

impl TryFrom<Guid> for CharacterId {
    type Error = TryFromWowIdError;

    fn try_from(value: Guid) -> Result<Self, Self::Error> {
        let e_type = (value.guid() >> 48) as u16;
        if e_type == EntityType::Player as u16 {
            Ok(CharacterId(value.guid() as u32))
        } else {
            Err(TryFromWowIdError::InvalidEntityType(e_type))
        }
    }
}

impl From<CharacterId> for Guid {
    fn from(id: CharacterId) -> Self {
        new_guid(EntityType::Player, id.0, 0)
    }
}

impl From<&CharacterId> for Guid {
    fn from(id: &CharacterId) -> Self {
        (*id).into()
    }
}

#[derive(Debug, Clone)]
pub struct Character {
    pub id: CharacterId,
    pub account: AccountId,
    pub name: String,
    pub level: Level,
    pub race: Race,
    pub class: Class,
    pub gender: Gender,

    pub skin_color: u8,
    pub face: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub facial_style: u8,

    pub zone: Area,
    pub map: Map,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

// position_x: -8949.94f32, //
// position_y: -132.50f32,  // human start zone
// position_z: 83.53f32,    //

#[derive(Debug, Clone)]
pub struct CharacterCreate {
    pub name: String,
    pub race: Race,
    pub class: Class,
    pub gender: Gender,

    pub skin_color: u8,
    pub face: u8,
    pub hair_style: u8,
    pub hair_color: u8,
    pub facial_style: u8,

    pub zone: u16,
    pub map: u16,
    pub position_x: f32,
    pub position_y: f32,
    pub position_z: f32,
}

#[derive(Debug, Clone)]
pub struct AccountDataCache {
    pub time: u32,
    pub data: Vec<u8>,
}

pub trait CharacterService {
    fn get(&self, id: CharacterId) -> impl Future<Output = Result<Option<Character>, CharacterServiceError>> + Send;
    fn get_by_name(&self, name: &str) -> impl Future<Output = Result<Option<Character>, CharacterServiceError>> + Send;
    /// Get list of friends, flags and notes for a given character
    fn get_friends(&self, id: CharacterId) -> impl Future<Output = Result<Vec<(CharacterId, u32, String)>, CharacterServiceError>> + Send;
    fn add_friend_by_name(
        &self,
        id: CharacterId,
        name: &str,
        note: Option<&str>,
    ) -> impl Future<Output = Result<Option<CharacterId>, CharacterServiceError>> + Send;
    fn list(&self) -> impl Future<Output = Result<Vec<Character>, CharacterServiceError>> + Send;
    fn get_by_account(&self, id: AccountId) -> impl Future<Output = Result<Vec<Character>, CharacterServiceError>> + Send;
    fn account_data(&self, id: AccountId) -> impl Future<Output = Result<Vec<(i16, AccountDataCache)>, CharacterServiceError>> + Send;
    fn character_account_data(&self, id: CharacterId) -> impl Future<Output = Result<Vec<(i16, AccountDataCache)>, CharacterServiceError>> + Send;
    fn count_by_account(&self, id: AccountId) -> impl Future<Output = Result<usize, CharacterServiceError>> + Send;
    fn name_available(&self, name: &str) -> impl Future<Output = Result<bool, CharacterServiceError>> + Send;
    fn create_character(&self, account: AccountId, create: CharacterCreate) -> impl Future<Output = Result<CharacterId, CharacterServiceError>> + Send;
    fn delete_character(&self, id: CharacterId) -> impl Future<Output = Result<(), CharacterServiceError>> + Send;
}

/// Errors that may occur when running character operations.
#[derive(Error, Debug)]
pub enum CharacterServiceError {
    #[error("no such account {0:?}")]
    NoSuchAccount(AccountId),
    #[error("no such character {0:?}")]
    NoSuchCharacter(CharacterId),
    #[error("persistence error {0:?}")]
    PersistError(String),
}
