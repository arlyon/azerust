#![feature(hash_drain_filter, arbitrary_enum_discriminant)]
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

use anyhow::{anyhow, Result};
use azerust_axum::api;
use azerust_game::accounts::AccountService;
use azerust_mysql_auth::{accounts::MySQLAccountService, realms::MySQLRealmList};
use azerust_utils::flatten;
use conf::AuthServerConfig;
use human_panic::setup_panic;
use sqlx::MySqlPool;
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
mod protocol;
mod wow_bincode;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic!();

    let opts = Opt::from_args();
    let config = AuthServerConfig::read(&opts.config).await?;

    let tracer = opentelemetry_jaeger::new_pipeline()
        .with_service_name("auth")
        .install_simple()?;

    let fmt_filter = std::env::var("RUST_LOG")
        .ok()
        .and_then(|log_filter| match log_filter.parse::<filter::Targets>() {
            Ok(targets) => Some(targets),
            Err(e) => {
                eprintln!(
                    "failed to parse filter environment variable `RUST_LOG={:?}`: {}",
                    log_filter, e
                );
                None
            }
        })
        .unwrap_or_else(|| {
            "error"
                .parse::<filter::Targets>()
                .expect("`error` filter should always parse successfully")
        });

    if let Some(port) = config.console_port {
        tracing_subscriber::registry()
            .with(
                console_subscriber::ConsoleLayer::builder()
                    .server_addr((config.bind_address, port))
                    .spawn(),
            )
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer().with_filter(fmt_filter))
            .init();
    } else {
        tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .with(tracing_subscriber::fmt::layer().with_filter(fmt_filter))
            .init();
    };

    match opts.command {
        Some(opt::OptCommand::Exec(c)) => match c {
            opt::Command::Account {
                command:
                    AccountCommand::Create {
                        username,
                        password,
                        email,
                    },
            } => {
                let pool = MySqlPool::connect(&config.auth_database).await?;
                let accounts = MySQLAccountService::new(pool);
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
    let pool = MySqlPool::connect(&auth_database).await?;

    let accounts = MySQLAccountService::new(pool.clone());
    let realms = MySQLRealmList::new(pool.clone(), Duration::from_secs(10));

    let server = AuthServer::new(accounts.clone(), realms.clone());

    let cancel = CancellationToken::new();

    let c2 = cancel.clone();
    ctrlc_async::set_handler(move || c2.cancel())?;

    if let Some(api_port) = api_port {
        let addr = SocketAddr::new(bind_address.into(), api_port);
        let api = flatten(
            tokio::task::Builder::new()
                .name("auth::graphql")
                .spawn(async move {
                    api(&addr, accounts.clone(), realms.clone())
                        .await
                        .map_err(|_| anyhow!("failed to start graphql api"))
                })?,
        );

        try_join!(
            server.start(bind_address, port, heartbeat_port, cancel),
            api
        )?;
    } else {
        server
            .start(bind_address, port, heartbeat_port, cancel)
            .await?;
    }

    Ok(())
}
