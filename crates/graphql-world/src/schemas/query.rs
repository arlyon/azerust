use std::marker::PhantomData;

use async_graphql::{Context, FieldResult, Object};
use azerust_game::characters::CharacterService;

use crate::models::Character;

pub struct Query<C> {
    character: PhantomData<C>,
}

impl<C> Default for Query<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> Query<C> {
    pub fn new() -> Self {
        Self { character: PhantomData }
    }
}

#[Object]
impl<C> Query<C>
where
    C: 'static + CharacterService + Send + Sync,
{
    async fn get_characters(&self, ctx: &Context<'_>) -> FieldResult<Vec<Character>> {
        let service = ctx.data::<C>()?;
        let accounts = service.list().await?;
        Ok(accounts.into_iter().map(Character).collect())
    }
}
