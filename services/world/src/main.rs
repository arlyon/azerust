#![feature(box_patterns)]
#![forbid(unsafe_code)]
#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    clippy::useless_conversion,
    clippy::unwrap_used,
    clippy::todo,
    clippy::unimplemented
)]

use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use anyhow::Result;
use azerust_axum::world_api;
use azerust_game::realms::RealmId;
use azerust_pg_auth::{PgAccountService, PgRealmList};
use azerust_pg_characters::PgCharacterService;
use azerust_utils::flatten;
use deadpool_postgres::{Runtime, tokio_postgres::NoTls};
use human_panic::setup_panic;
use structopt::StructOpt;
use tokio::try_join;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use tracing_subscriber::{Layer, filter, prelude::*};

use crate::{conf::WorldServerConfig, opt::Opt, worldserver::WorldServer};

mod client;
mod conf;
mod opt;
mod world;
mod worldserver;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic!();

    let opts: Opt = Opt::from_args();
    let config = WorldServerConfig::read(&opts.config).await?;

    // let tracer = opentelemetry_jaeger::new_pipeline().with_service_name("auth").install_simple()?;

    let fmt_filter = std::env::var("RUST_LOG")
        .ok()
        .and_then(|log_filter| match log_filter.parse::<filter::Targets>() {
            Ok(targets) => Some(targets),
            Err(e) => {
                eprintln!("failed to parse filter environment variable `RUST_LOG={:?}`: {}", log_filter, e);
                None
            }
        })
        .unwrap_or_else(|| "error".parse::<filter::Targets>().expect("`error` filter should always parse successfully"));

    if let Some(_port) = config.console_port {
        tracing_subscriber::registry()
            // .with(console_subscriber::ConsoleLayer::builder().server_addr((config.bind_address, port)).spawn())
            // .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer().with_filter(fmt_filter))
            .init();
    } else {
        tracing_subscriber::registry()
            // .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer().with_filter(fmt_filter))
            .init();
    };

    match opts.command {
        Some(opt::OptCommand::Init) => {
            let auth = WorldServerConfig {
                bind_address: "0.0.0.0".parse::<Ipv4Addr>().expect("Valid IP"),
                port: 3724,
                console_port: None,
                api_port: None,
                auth_server_address: "localhost:1234".to_string(),

                realm_id: RealmId(1),
                data_dir: 0,

                character_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
                auth_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
                world_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            };
            auth.write(&opts.config).await?;
        }
        None => match start_server(config).await {
            Ok(_) => {}
            Err(e) => error!("server shutting down: {}", e),
        },
    };

    info!("uploading spans");
    opentelemetry::global::shutdown_tracer_provider(); // export remaining spans

    Ok(())
}

async fn start_server(config: WorldServerConfig) -> Result<()> {
    let mut pool_cfg = deadpool_postgres::Config::new();
    pool_cfg.url = Some(config.auth_database.clone());
    pool_cfg.connect_timeout = Some(Duration::from_secs(3));
    let auth_pool = pool_cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let mut pool_cfg = deadpool_postgres::Config::new();
    pool_cfg.url = Some(config.character_database.clone());
    pool_cfg.connect_timeout = Some(Duration::from_secs(3));
    let character_pool = pool_cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    debug!("Loaded config {:?}", config);

    let accounts = PgAccountService::new(auth_pool.clone());
    let realms = PgRealmList::new(auth_pool.clone(), Duration::from_secs(60));
    let characters = PgCharacterService::new(character_pool.clone());

    let server = WorldServer::new(config.realm_id, accounts, realms, characters.clone(), config.auth_server_address);

    let cancel = CancellationToken::new();

    let c2 = cancel.clone();
    ctrlc_async::set_handler(move || c2.cancel())?;

    if let Some(api_port) = config.api_port {
        let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), api_port);
        let api = flatten(
            tokio::task::Builder::new()
                .name("auth::graphql")
                .spawn(async move { world_api(&addr, characters).await.map_err(|_| anyhow::anyhow!("failed to start graphql api")) })?,
        );

        try_join!(server.start(cancel), api)?;
    } else {
        server.start(cancel).await?;
    }

    Ok(())
}
