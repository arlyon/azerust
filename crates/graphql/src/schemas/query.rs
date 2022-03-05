use std::marker::PhantomData;

use async_graphql::{Context, FieldResult, Object};
use azerust_game::{accounts::AccountService, realms::RealmList};

use crate::models::{Account, Realm};

pub struct Query<A, R> {
    account: PhantomData<A>,
    realm: PhantomData<R>,
}

impl<A, R> Query<A, R> {
    pub fn new() -> Self {
        Self {
            account: PhantomData,
            realm: PhantomData,
        }
    }
}

#[Object]
impl<A, R> Query<A, R>
where
    A: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
{
    async fn get_users(&self, ctx: &Context<'_>) -> FieldResult<Vec<Account>> {
        let service = ctx.data::<A>()?;
        let accounts = service.list_account().await?;
        Ok(accounts.into_iter().map(Account).collect())
    }

    async fn get_user(&self, ctx: &Context<'_>, username: String) -> FieldResult<Option<Account>> {
        let service = ctx.data::<A>()?;
        let account = service.get_by_username(&username).await?;
        Ok(account.map(|a| Account(a)))
    }

    async fn get_realms(&self, ctx: &Context<'_>) -> FieldResult<Vec<Realm>> {
        let service = ctx.data::<R>()?;
        Ok(service.realms().await.into_iter().map(Realm).collect())
    }
}
