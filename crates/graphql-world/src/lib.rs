use async_graphql::{EmptySubscription, Schema};
use azerust_game::characters::CharacterService;
pub use schemas::{Mutation, Query};

mod models;
mod schemas;

pub fn create_schema<C: 'static + CharacterService + Send + Sync>(
    characters: C,
) -> Schema<Query<C>, Mutation<C>, EmptySubscription> {
    Schema::build(Query::new(), Mutation::new(), EmptySubscription)
        .data(characters)
        .finish()
}
