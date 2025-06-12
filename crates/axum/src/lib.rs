mod auth;
mod world;


use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
pub use auth::auth_api;
use axum::response::{self, IntoResponse};
pub use world::world_api;

async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}
