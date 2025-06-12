#![feature(hash_extract_if)]
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

use anyhow::{Result, anyhow};
use azerust_axum::auth_api;
use azerust_game::accounts::AccountService;
use azerust_pg_auth::{PgAccountService, PgRealmList};
use azerust_utils::flatten;
use conf::AuthServerConfig;
use deadpool_postgres::{Runtime, tokio_postgres::NoTls};
use human_panic::setup_panic;
use structopt::StructOpt;
use tokio::try_join;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{filter, prelude::*};

use crate::{
    authserver::AuthServer,
    opt::{AccountCommand, Opt},
};

mod authserver;
mod conf;
mod opt;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic!();

    let opts = Opt::from_args();
    let config = AuthServerConfig::read(&opts.config).await?;

    // let tracer = opentelemetry_jaeger::new_pipeline()
    //     .with_service_name("auth")
    //     .install_simple()?;

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
        Some(opt::OptCommand::Exec(c)) => match c {
            opt::Command::Account {
                command: AccountCommand::Create { username, password, email },
            } => {
                let mut pool_cfg = deadpool_postgres::Config::new();
                pool_cfg.url = Some(config.auth_database);
                pool_cfg.connect_timeout = Some(Duration::from_secs(1));
                let pool = pool_cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
                let accounts = PgAccountService::new(pool);
                match accounts.create_account(&username, &password, &email).await {
                    Ok(id) => println!("created account {id}"),
                    Err(e) => eprintln!("failed to create account: {e}"),
                };
            }
        },
        Some(opt::OptCommand::Init) => {
            let auth = AuthServerConfig {
                bind_address: "0.0.0.0".parse::<Ipv4Addr>().expect("Valid IP"),
                port: 3724,
                heartbeat_port: 1234,
                api_port: None,
                console_port: None,
                auth_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
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

async fn start_server(
    AuthServerConfig {
        bind_address,
        api_port,
        heartbeat_port,
        port,
        auth_database,
        ..
    }: AuthServerConfig,
) -> Result<()> {
    let mut pool_cfg = deadpool_postgres::Config::new();
    pool_cfg.url = Some(auth_database.clone());
    pool_cfg.connect_timeout = Some(Duration::from_secs(3));
    let pool = pool_cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let accounts = PgAccountService::new(pool.clone());
    let realms = PgRealmList::new(pool.clone(), Duration::from_secs(10));

    let server = AuthServer::new(accounts.clone(), realms.clone());

    let cancel = CancellationToken::new();

    let c2 = cancel.clone();
    ctrlc_async::set_handler(move || c2.cancel())?;

    if let Some(api_port) = api_port {
        let addr = SocketAddr::new(bind_address.into(), api_port);
        let api = flatten(tokio::task::Builder::new().name("auth::graphql").spawn(async move {
            auth_api(&addr, accounts.clone(), realms.clone())
                .await
                .map_err(|_| anyhow!("failed to start graphql api"))
        })?);

        try_join!(server.start(bind_address, port, heartbeat_port, cancel), api)?;
    } else {
        server.start(bind_address, port, heartbeat_port, cancel).await?;
    }

    Ok(())
}
