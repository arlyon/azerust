use async_trait::async_trait;
use sqlx::Type;

use crate::accounts::AccountId;

#[derive(Debug, Clone, Copy, Type)]
#[sqlx(transparent)]
pub struct CharacterId(u32);

#[derive(Debug, Clone)]
pub struct Character {
    pub id: CharacterId,
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

#[async_trait]
pub trait CharacterService {
    async fn get(&self, id: CharacterId) -> Result<Character, ()>;
    async fn get_by_account(&self, id: AccountId) -> Result<Vec<Character>, ()>;
    async fn count_by_account(&self, id: AccountId) -> Result<usize, ()>;
    async fn name_available(&self, name: String) -> Result<bool, ()>;
    async fn create_character(&self, account: AccountId) -> Result<(), ()>;
    async fn delete_character(&self, id: CharacterId) -> Result<(), ()>;
}
