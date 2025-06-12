use std::net::SocketAddr;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Router, extract::Extension, routing::get};
use azerust_game::characters::CharacterService;

use crate::graphql_playground;

async fn world_handler<C: 'static + CharacterService + Send + Sync>(
    schema: Extension<Schema<azerust_graphql_world::Query<C>, azerust_graphql_world::Mutation<C>, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn world_api<C: 'static + CharacterService + Send + Sync>(listen_addr: &SocketAddr, characters: C) -> Result<(), ()> {
    let schema = azerust_graphql_world::create_schema(characters);

    let app = Router::new()
        .route("/", get(graphql_playground).post(world_handler::<C>))
        .layer(Extension(schema));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
