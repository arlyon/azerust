use async_trait::async_trait;
use azerust_game::{
    accounts::AccountId,
    characters::{Character, CharacterId, CharacterService},
};
use sqlx::{query, query_as, MySqlPool};
use tracing::debug;

pub struct MySQLCharacterService {
    pool: MySqlPool,
}

impl MySQLCharacterService {
    pub fn new(pool: MySqlPool) -> Self {
        debug!("Starting character service");
        Self { pool }
    }
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
        query_as!(
            Character,
            "SELECT guid as 'id: _', account as 'account: _', name, level, race, class, gender, skin as skin_color, face, hairStyle as hair_style, hairColor as hair_color, facialStyle as facial_style, zone, map, position_x, position_y, position_z FROM characters where account = ?",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|_| ())
    }

    async fn count_by_account(&self, id: AccountId) -> Result<usize, ()> {
        query!("SELECT count(*) as c FROM characters where account = ?", id)
            .fetch_one(&self.pool)
            .await
            .map(|c| c.c as usize)
            .map_err(|_| ())
    }

    async fn name_available(&self, name: String) -> Result<bool, ()> {
        query!("SELECT count(*) as c FROM characters where name = ?", name)
            .fetch_one(&self.pool)
            .await
            .map(|c| c.c == 0)
            .map_err(|_| ())
    }

    async fn create_character(&self, account: AccountId) -> Result<(), ()> {
        todo!()
    }

    async fn delete_character(&self, id: CharacterId) -> Result<(), ()> {
        query!("DELETE FROM characters where guid = ?", id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| ())
    }
}
