use std::marker::PhantomData;

use async_graphql::{Context, FieldResult, InputObject, Object};
use azerust_game::accounts::AccountService;

pub struct Mutation<T> {
    marker: PhantomData<T>,
}

#[derive(InputObject)]
struct UserCreate {
    username: String,
    email: String,
    password: String,
}

impl<T> Mutation<T> {
    pub fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

#[Object]
impl<T> Mutation<T>
where
    T: 'static + AccountService + Send + Sync,
{
    /// Creates a new user account.
    async fn register_user(&self, ctx: &Context<'_>, user: UserCreate) -> FieldResult<u32> {
        let service = ctx.data::<T>()?;
        let id = service
            .create_account(&user.username, &user.password, &user.email)
            .await?;
        Ok(id.0)
    }
}
