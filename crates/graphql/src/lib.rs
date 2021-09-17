use async_graphql::{EmptySubscription, Schema};
use game::{accounts::AccountService, realms::RealmList};
use schemas::{Mutation, Query};

mod models;
mod schemas;

pub fn create_schema<
    A: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
>(
    accounts: A,
    realms: R,
) -> Schema<Query<A, R>, Mutation<A>, EmptySubscription> {
    Schema::build(Query::new(), Mutation::new(), EmptySubscription)
        .data(accounts)
        .data(realms)
        .finish()
}