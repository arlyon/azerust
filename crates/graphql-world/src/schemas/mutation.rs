use std::marker::PhantomData;

use async_graphql::{Context, FieldResult, InputObject, Object};
use azerust_game::{
    accounts::AccountId,
    characters::{CharacterCreate, CharacterService, Class, Gender, Race},
};

pub struct Mutation<C> {
    marker: PhantomData<C>,
}

#[derive(InputObject)]
struct UserCreate {
    username: String,
    email: String,
    password: String,
}

impl<C> Default for Mutation<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Mutation<C> {
    pub fn new() -> Self {
        Self { marker: PhantomData }
    }
}

#[Object]
impl<C> Mutation<C>
where
    C: 'static + CharacterService + Send + Sync,
{
    /// Creates a new user account.
    async fn create_character(&self, ctx: &Context<'_>, account: u32, name: String) -> FieldResult<u32> {
        let characters = ctx.data::<C>()?;
        let id = characters
            .create_character(AccountId(account as u32), CharacterCreate {
                class: Class::Warrior,
                face: 0,
                facial_style: 0,
                hair_color: 0,
                hair_style: 0,
                skin_color: 0,
                gender: Gender::Male,
                map: 0,               //
                zone: 1,              //
                position_x: -6240.32, // dwarf start zone
                position_y: 331.033,  //
                position_z: 382.758,  //
                name: name.clone(),
                race: Race::Human,
            })
            .await?;

        Ok(id.0)
    }
}

#[derive(InputObject)]
struct BanDuration {
    days: u64,
}
