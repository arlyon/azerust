use azerust_game::{
    EntityType,
    accounts::AccountId,
    characters::{AccountDataCache, Character, CharacterCreate, CharacterId, CharacterService, CharacterServiceError},
    new_guid,
};
use rand::Rng;
use tracing::{debug, instrument};

use crate::cornucopia::queries::character;

#[derive(Clone)]
pub struct PgCharacterService {
    pool: deadpool_postgres::Pool,
}

impl PgCharacterService {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        debug!("Starting character service");
        Self { pool }
    }
}

impl CharacterService for PgCharacterService {
    async fn get(&self, id: CharacterId) -> Result<Option<Character>, CharacterServiceError> {
        character::get_character_by_id()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .map(|c| Character {
                id: CharacterId(c.id),
                account: AccountId(c.account),
                class: c.class.try_into().unwrap(),
                face: c.face.try_into().unwrap(),
                facial_style: c.facial_style.try_into().unwrap(),
                gender: c.gender.try_into().unwrap(),
                hair_color: c.hair_color.try_into().unwrap(),
                hair_style: c.hair_style.try_into().unwrap(),
                level: c.level.into(),
                map: c.map.try_into().unwrap(),
                name: c.name.to_owned(),
                position_x: c.position_x,
                position_y: c.position_y,
                position_z: c.position_z,
                race: c.race.try_into().unwrap(),
                skin_color: c.skin_color.try_into().unwrap(),
                zone: c.zone.try_into().unwrap(),
            })
            .opt()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Character>, CharacterServiceError> {
        character::get_character_by_name()
            .bind(&self.pool.get().await.unwrap(), &name)
            .map(|c| Character {
                id: CharacterId(c.id),
                account: AccountId(c.account),
                class: c.class.try_into().unwrap(),
                face: c.face.try_into().unwrap(),
                facial_style: c.facial_style.try_into().unwrap(),
                gender: c.gender.try_into().unwrap(),
                hair_color: c.hair_color.try_into().unwrap(),
                hair_style: c.hair_style.try_into().unwrap(),
                level: c.level.into(),
                map: c.map.try_into().unwrap(),
                name: c.name.to_owned(),
                position_x: c.position_x,
                position_y: c.position_y,
                position_z: c.position_z,
                race: c.race.try_into().unwrap(),
                skin_color: c.skin_color.try_into().unwrap(),
                zone: c.zone.try_into().unwrap(),
            })
            .opt()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn get_friends(&self, id: CharacterId) -> Result<Vec<(CharacterId, u32, String)>, CharacterServiceError> {
        character::get_friends()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .map(|r| (CharacterId(r.friend), r.flags, r.note.to_owned()))
            .all()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn add_friend_by_name(&self, id: CharacterId, name: &str, note: Option<&str>) -> Result<Option<CharacterId>, CharacterServiceError> {
        let Some(other) = self.get_by_name(name).await.unwrap() else {
            return Ok(None);
        };

        character::add_friend_by_name()
            .bind(&self.pool.get().await.unwrap(), &id.0, &other.id.0, &note)
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
            .and(Ok(Some(other.id)))
    }

    #[instrument(skip(self))]
    async fn get_by_account(&self, id: AccountId) -> Result<Vec<Character>, CharacterServiceError> {
        character::get_character_by_account()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .map(|c| Character {
                id: CharacterId(c.id),
                account: AccountId(c.account),
                class: c.class.try_into().unwrap(),
                face: c.face.try_into().unwrap(),
                facial_style: c.facial_style.try_into().unwrap(),
                gender: c.gender.try_into().unwrap(),
                hair_color: c.hair_color.try_into().unwrap(),
                hair_style: c.hair_style.try_into().unwrap(),
                level: c.level.into(),
                map: c.map.try_into().unwrap(),
                name: c.name.to_owned(),
                position_x: c.position_x,
                position_y: c.position_y,
                position_z: c.position_z,
                race: c.race.try_into().unwrap(),
                skin_color: c.skin_color.try_into().unwrap(),
                zone: c.zone.try_into().unwrap(),
            })
            .all()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn count_by_account(&self, id: AccountId) -> Result<usize, CharacterServiceError> {
        character::count_by_account()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .one()
            .await
            .map(|c| c as usize)
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn name_available(&self, name: &str) -> Result<bool, CharacterServiceError> {
        let avaialble = character::name_available()
            .bind(&self.pool.get().await.unwrap(), &name)
            .one()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))?;

        tracing::trace!("name available: {avaialble}");
        Ok(avaialble)
    }

    #[instrument(skip(self))]
    async fn create_character(&self, account: AccountId, create: CharacterCreate) -> Result<CharacterId, CharacterServiceError> {
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
            new_guid(EntityType::Player, rng.r#gen(), 0)
        };

        tracing::info!("creating character {name}");

        // todo taximask, taxi_path, exploredZones, equipmentCache, knownTitles

        character::insert_character()
            .bind(
                &self.pool.get().await.unwrap(),
                &account.0,
                &CharacterId::try_from(id).unwrap().0,
                &name,
                &race.as_int(),
                &class.as_int(),
                &gender.as_int(),
                &skin_color,
                &face,
                &hair_style,
                &hair_color,
                &facial_style,
                &(zone),
                &(map),
                &position_x,
                &position_y,
                &position_z,
            )
            .one()
            .await
            .map(CharacterId)
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn delete_character(&self, id: CharacterId) -> Result<(), CharacterServiceError> {
        character::delete_character()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .await
            .map(|_| ())
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn account_data(&self, id: AccountId) -> Result<Vec<(i16, AccountDataCache)>, CharacterServiceError> {
        character::account_data()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .map(|r| {
                (r.r#type, AccountDataCache {
                    time: r.time,
                    data: r.data.to_owned(),
                })
            })
            .all()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn character_account_data(&self, id: CharacterId) -> Result<Vec<(i16, AccountDataCache)>, CharacterServiceError> {
        character::character_account_data()
            .bind(&self.pool.get().await.unwrap(), &id.0)
            .map(|r| {
                (r.r#type, AccountDataCache {
                    time: r.time,
                    data: r.data.to_owned(),
                })
            })
            .all()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }

    async fn list(&self) -> Result<Vec<Character>, CharacterServiceError> {
        character::list_characters()
            .bind(&self.pool.get().await.unwrap())
            .map(|c| Character {
                id: CharacterId(c.id),
                account: AccountId(c.account),
                class: c.class.try_into().unwrap(),
                face: c.face.try_into().unwrap(),
                facial_style: c.facial_style.try_into().unwrap(),
                gender: c.gender.try_into().unwrap(),
                hair_color: c.hair_color.try_into().unwrap(),
                hair_style: c.hair_style.try_into().unwrap(),
                level: c.level.into(),
                map: c.map.try_into().unwrap(),
                name: c.name.to_owned(),
                position_x: c.position_x,
                position_y: c.position_y,
                position_z: c.position_z,
                race: c.race.try_into().unwrap(),
                skin_color: c.skin_color.try_into().unwrap(),
                zone: c.zone.try_into().unwrap(),
            })
            .all()
            .await
            .map_err(|e| CharacterServiceError::PersistError(e.to_string()))
    }
}
