use std::marker::PhantomData;

use async_graphql::{Context, FieldResult, InputObject, Object};
use azerust_game::{
    accounts::{AccountId, AccountService},
    characters::{CharacterCreate, CharacterService},
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

impl<C> Mutation<C> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

#[Object]
impl<C> Mutation<C>
where
    C: 'static + CharacterService + Send + Sync,
{
    /// Creates a new user account.
    async fn create_character(&self, ctx: &Context<'_>) -> FieldResult<u32> {
        todo!();
    }
}

#[derive(InputObject)]
struct BanDuration {
    days: u64,
}
