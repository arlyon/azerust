use async_trait::async_trait;
use azerust_game::{
    accounts::AccountId,
    characters::{Character, CharacterId, CharacterService},
};
use sqlx::{query_as, MySqlPool};

pub struct MySQLCharacterService {
    pool: MySqlPool,
}

#[async_trait]
impl CharacterService for MySQLCharacterService {
    async fn get(&self, id: CharacterId) -> Result<Character, ()> {
        query_as!(
            Character,
            "SELECT guid as 'id: _', account as 'account: _', name, level, race, class, gender, skin as skin_color, face, hairStyle as hair_style, hairColor as hair_color, facialStyle as facial_style, zone, map, position_x, position_y, position_z FROM characters where guid = ?",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| ())
    }

    async fn get_by_account(&self, id: AccountId) -> Result<Vec<Character>, ()> {
        todo!()
    }

    async fn count_by_account(&self, id: AccountId) -> Result<usize, ()> {
        todo!()
    }

    async fn name_available(&self, name: String) -> Result<bool, ()> {
        todo!()
    }

    async fn create_character(&self, account: AccountId) -> Result<(), ()> {
        todo!()
    }

    async fn delete_character(&self, id: CharacterId) -> Result<(), ()> {
        todo!()
    }
}
