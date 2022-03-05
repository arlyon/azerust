use std::{marker::PhantomData, time::Duration};

use async_graphql::{Context, FieldResult, InputObject, Object};
use azerust_game::accounts::{AccountId, AccountService};

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

    async fn set_ban_status(
        &self,
        ctx: &Context<'_>,
        id: u32,
        duration: Option<BanDuration>,
        reason: Option<String>,
    ) -> FieldResult<bool> {
        let service = ctx.data::<T>()?;
        service
            .set_ban(
                AccountId(id),
                "arlyon",
                duration.map(|d| Duration::from_secs(d.days * 86400)),
                reason.as_deref(),
            )
            .await?;
        Ok(true)
    }
}

#[derive(InputObject)]
struct BanDuration {
    days: u64,
}
