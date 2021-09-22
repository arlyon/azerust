use std::collections::HashMap;

use async_trait::async_trait;
use azerust_game::{
    accounts::AccountId,
    characters::{
        AccountData, AccountDataCache, Character, CharacterCreate, CharacterId, CharacterService,
        CharacterServiceError, DualDataCache,
    },
    EntityType, WowId,
};
use rand::Rng;
use sqlx::{query, query_as, MySqlPool};
use tracing::{debug, instrument};

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
    async fn get(&self, id: CharacterId) -> Result<Character, CharacterServiceError> {
        query_as!(
            Character,
            "SELECT guid as 'id: _', account as 'account: _', name, level, race, class, gender, skin as skin_color, face, hairStyle as hair_style, hairColor as hair_color, facialStyle as facial_style, zone, map, position_x, position_y, position_z FROM characters where guid = ?",
            id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn get_by_account(&self, id: AccountId) -> Result<Vec<Character>, CharacterServiceError> {
        query_as!(
            Character,
            "SELECT guid as 'id: _', account as 'account: _', name, level, race, class, gender, skin as skin_color, face, hairStyle as hair_style, hairColor as hair_color, facialStyle as facial_style, zone, map, position_x, position_y, position_z FROM characters where account = ?",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn count_by_account(&self, id: AccountId) -> Result<usize, CharacterServiceError> {
        query!("SELECT count(*) as c FROM characters where account = ?", id)
            .fetch_one(&self.pool)
            .await
            .map(|c| c.c as usize)
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn name_available(&self, name: &str) -> Result<bool, CharacterServiceError> {
        query!("SELECT count(*) as c FROM characters where name = ?", name)
            .fetch_one(&self.pool)
            .await
            .map(|c| c.c == 0)
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn create_character(
        &self,
        account: AccountId,
        create: CharacterCreate,
    ) -> Result<(), CharacterServiceError> {
        let CharacterCreate {
            name,
            race,
            class,
            gender,
            face,
            facial_style,
            hair_color,
            hair_style,
            map,
            position_x,
            position_y,
            position_z,
            skin_color,
            zone,
        } = create;
        let id = {
            let mut rng = rand::thread_rng();
            WowId::new(EntityType::Player, rng.gen(), 0)
        };

        // todo taximask, taxi_path, exploredZones, equipmentCache, knownTitles

        query!(
            "INSERT INTO characters (account, guid, level, name, race, class, gender, skin, face, hairStyle, hairColor, facialStyle, zone, map, position_x, position_y, position_z, taximask, taxi_path, exploredZones, equipmentCache, knownTitles) values (?, ?, 1, ?,?,?,?, ?, ?, ?, ?, ?, ?,?,?,?,?, '','', '', '', '')", 
            account, id, name, race, class, gender, skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn delete_character(&self, id: CharacterId) -> Result<(), CharacterServiceError> {
        query!("DELETE FROM characters where guid = ?", id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn account_data(&self, id: AccountId) -> Result<AccountData, CharacterServiceError> {
        let mut rows = query!(
            "SELECT type, time, data FROM account_data WHERE accountId = ?",
            id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CharacterServiceError::PersistError(e.to_string()))?
        .into_iter()
        .map(|r| {
            (
                r.r#type,
                AccountDataCache {
                    time: r.time,
                    data: r.data,
                },
            )
        })
        .collect::<HashMap<_, _>>();

        Ok(AccountData {
            config: DualDataCache {
                global: rows.remove(&0),
                per_char: rows.remove(&1),
            },
            bindings: DualDataCache {
                global: rows.remove(&2),
                per_char: rows.remove(&3),
            },
            macros: DualDataCache {
                global: rows.remove(&4),
                per_char: rows.remove(&5),
            },
            per_char_layout: rows.remove(&6),
            per_char_chat: rows.remove(&7),
        })
    }
}
