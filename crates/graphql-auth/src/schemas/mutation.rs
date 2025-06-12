use std::{marker::PhantomData, net::IpAddr, str::FromStr, time::Duration};

use async_graphql::{Context, FieldResult, InputObject, Object};
use azerust_game::{
    accounts::{AccountId, AccountService},
    realms::RealmList,
};

use crate::models::Realm;

pub struct Mutation<A, R> {
    _accounts: PhantomData<A>,
    _realms: PhantomData<R>,
}

#[derive(InputObject)]
struct UserCreate {
    username: String,
    email: String,
    password: String,
}

impl<A, R> Default for Mutation<A, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A, R> Mutation<A, R> {
    pub fn new() -> Self {
        Self {
            _accounts: PhantomData,
            _realms: PhantomData,
        }
    }
}

#[Object]
impl<A, R> Mutation<A, R>
where
    A: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
{
    /// Creates a new user account.
    async fn register_user(&self, ctx: &Context<'_>, user: UserCreate) -> FieldResult<u32> {
        let service = ctx.data::<A>()?;
        let id = service.create_account(&user.username, &user.password, &user.email).await?;
        Ok(id.0)
    }

    async fn set_ban_status(&self, ctx: &Context<'_>, id: u32, duration: Option<BanDuration>, reason: Option<String>) -> FieldResult<bool> {
        let service = ctx.data::<A>()?;
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

    async fn create_realm(
        &self,
        ctx: &Context<'_>,
        name: String,
        address: String,
        local_address: String,
        local_subnet_mask: String,
        port: u16,
    ) -> FieldResult<Realm> {
        let realms = ctx.data::<R>()?;

        let address = IpAddr::from_str(&address)?;
        let local_address = IpAddr::from_str(&local_address)?;
        let local_subnet_mask = IpAddr::from_str(&local_subnet_mask)?;

        let realm = realms.create_realm(&name, &address, &local_address, &local_subnet_mask, port).await?;

        Ok(Realm(realm))
    }
}

#[derive(InputObject)]
struct BanDuration {
    days: u64,
}
