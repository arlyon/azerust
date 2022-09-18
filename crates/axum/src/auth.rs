use std::net::SocketAddr;

use async_graphql::{
    futures_util::try_join,
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

async fn auth_handler<
    A: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
>(
    schema: Extension<
        Schema<
            azerust_graphql_auth::Query<A, R>,
            azerust_graphql_auth::Mutation<A>,
            EmptySubscription,
        >,
    >,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn auth_api<
    T: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
>(
    listen_addr: &SocketAddr,
    account: T,
    realms: R,
) -> anyhow::Result<()> {
    let schema = azerust_graphql_auth::create_schema(account, realms);

    let app = Router::new()
        .route("/", get(graphql_playground).post(auth_handler::<T, R>))
        .layer(Extension(schema));

    let auth_server = Server::bind(listen_addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
