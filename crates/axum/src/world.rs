use std::net::SocketAddr;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};
use azerust_game::{accounts::AccountService, characters::CharacterService, realms::RealmList};

use crate::graphql_playground;

async fn world_handler<C: 'static + CharacterService + Send + Sync>(
    schema: Extension<
        Schema<
            azerust_graphql_world::Query<C>,
            azerust_graphql_world::Mutation<C>,
            EmptySubscription,
        >,
    >,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn world_api<C: 'static + CharacterService + Send + Sync>(
    listen_addr: &SocketAddr,
    characters: C,
) -> Result<(), ()> {
    let schema = azerust_graphql_world::create_schema(characters);

    let app = Router::new()
        .route("/", get(graphql_playground).post(world_handler::<C>))
        .layer(Extension(schema));

    Server::bind(listen_addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| ())?;

    Ok(())
}
