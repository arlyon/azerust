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
    AddExtensionLayer, Router, Server,
};
use azerust_game::{accounts::AccountService, realms::RealmList};
use azerust_graphql::{create_schema, Mutation, Query};

async fn graphql_handler<
    A: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
>(
    schema: Extension<Schema<Query<A, R>, Mutation<A>, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    response::Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

pub async fn api<
    T: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
>(
    listen_addr: &SocketAddr,
    account: T,
    realms: R,
) -> Result<(), ()> {
    let schema = create_schema(account, realms);

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler::<T, R>))
        .layer(AddExtensionLayer::new(schema));

    Server::bind(listen_addr)
        .serve(app.into_make_service())
        .await
        .map_err(|_| ())?;

    Ok(())
}
