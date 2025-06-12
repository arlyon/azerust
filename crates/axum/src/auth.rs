use std::net::SocketAddr;

use async_graphql::{EmptySubscription, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{Router, extract::Extension, routing::get};
use azerust_game::{accounts::AccountService, realms::RealmList};

use crate::graphql_playground;

async fn auth_handler<A: 'static + AccountService + Send + Sync, R: 'static + RealmList + Send + Sync>(
    schema: Extension<Schema<azerust_graphql_auth::Query<A, R>, azerust_graphql_auth::Mutation<A, R>, EmptySubscription>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

pub async fn auth_api<T: 'static + AccountService + Send + Sync, R: 'static + RealmList + Send + Sync>(
    listen_addr: &SocketAddr,
    account: T,
    realms: R,
) -> anyhow::Result<()> {
    let schema = azerust_graphql_auth::create_schema(account, realms);

    let app = Router::new()
        .route("/", get(graphql_playground).post(auth_handler::<T, R>))
        .layer(Extension(schema));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(listen_addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
