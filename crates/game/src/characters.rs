use async_trait::async_trait;
use sqlx::Type;
use thiserror::Error;

use crate::{accounts::AccountId, EntityType, WowId};

#[derive(Error, Debug, Clone, Copy)]
pub enum TryFromWowIdError {
    #[error("invalid entity {0}")]
    InvalidEntityType(u16),
}

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(transparent)]
pub struct CharacterId(u64);

impl TryFrom<WowId> for CharacterId {
    type Error = TryFromWowIdError;

    fn try_from(value: WowId) -> Result<Self, Self::Error> {
        let e_type = (value.0 >> 48) as u16;
        if e_type == EntityType::Player as u16 {
            Ok(CharacterId(value.0))
        } else {
            Err(TryFromWowIdError::InvalidEntityType(e_type))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Character {
    pub id: WowId,
    pub account: AccountId,
    pub name: String,
    pub level: u8,
    pub race: u8,
    pub class: u8,
    pub gender: u8,

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

// position_x: -8949.94f32, //
// position_y: -132.50f32,  // human start zone
// position_z: 83.53f32,    //

#[derive(Debug, Clone)]
pub struct CharacterCreate {
    pub name: String,
    pub race: u8,
    pub class: u8,
    pub gender: u8,

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

#[derive(Debug, Default)]
pub struct DualDataCache {
    pub global: Option<AccountDataCache>,
    pub per_char: Option<AccountDataCache>,
}

#[derive(Debug, Default)]
pub struct AccountData {
    pub config: DualDataCache,
    pub bindings: DualDataCache,
    pub macros: DualDataCache,
    pub per_char_layout: Option<AccountDataCache>,
    pub per_char_chat: Option<AccountDataCache>,
}

impl AccountData {
    pub fn items(self) -> [Option<AccountDataCache>; 8] {
        [
            self.config.global,
            self.config.per_char,
            self.bindings.global,
            self.bindings.per_char,
            self.macros.global,
            self.macros.per_char,
            self.per_char_layout,
            self.per_char_chat,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct AccountDataCache {
    pub time: u32,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait CharacterService {
    async fn get(&self, id: CharacterId) -> Result<Character, CharacterServiceError>;
    async fn get_by_account(&self, id: AccountId) -> Result<Vec<Character>, CharacterServiceError>;
    async fn account_data(&self, id: AccountId) -> Result<AccountData, CharacterServiceError>;
    async fn count_by_account(&self, id: AccountId) -> Result<usize, CharacterServiceError>;
    async fn name_available(&self, name: &String) -> Result<bool, CharacterServiceError>;
    async fn create_character(
        &self,
        account: AccountId,
        create: CharacterCreate,
    ) -> Result<(), CharacterServiceError>;
    async fn delete_character(&self, id: CharacterId) -> Result<(), CharacterServiceError>;
}

/// Errors that may occur when running account operations.
#[derive(Error, Debug)]
pub enum CharacterServiceError {
    #[error("no such account {0:?}")]
    NoSuchAccount(AccountId),
    #[error("no such character {0:?}")]
    NoSuchCharacter(CharacterId),
    #[error("persistence error {0:?}")]
    PersistError(String),
}
