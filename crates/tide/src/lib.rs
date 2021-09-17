use std::io;

use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use azerust_game::{accounts::AccountService, realms::RealmList};
use azerust_graphql::create_schema;
use tide::{http::mime, listener::ToListener, Body, Response, StatusCode};

pub async fn api<
    L,
    T: 'static + AccountService + Send + Sync,
    R: 'static + RealmList + Send + Sync,
>(
    listen_addr: L,
    account: T,
    realms: R,
) -> io::Result<()>
where
    L: ToListener<()>,
{
    let mut app = tide::new();
    let schema = create_schema(account, realms);

    app.at("/graphql")
        .post(async_graphql_tide::endpoint(schema));

    app.at("/").get(|_| async move {
        Ok(Response::builder(StatusCode::Ok)
            .body(Body::from_string(playground_source(
                // note that the playground needs to know
                // the path to the graphql endpoint
                GraphQLPlaygroundConfig::new("/graphql"),
            )))
            .content_type(mime::HTML)
            .build())
    });

    Ok(app.listen(listen_addr).await?)
}
