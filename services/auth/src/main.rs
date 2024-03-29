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
    if let Some(port) = config.console_port {
        console_subscriber::ConsoleLayer::builder()
            .server_addr((config.bind_address, port))
            .init();
    } else {
        tracing_subscriber::fmt::init();
    }

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
        None => start_server(config).await?,
    };

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

    if let Some(api_port) = api_port {
        let addr = SocketAddr::new(bind_address.into(), api_port);
        let api = flatten(
            tokio::task::Builder::new()
                .name("auth::graphql")
                .spawn(async move {
                    api(&addr, accounts.clone(), realms.clone())
                        .await
                        .map_err(|_| anyhow!("failed to start graphql api"))
                }),
        );

        try_join!(server.start(bind_address, port, heartbeat_port), api)?;
    } else {
        server.start(bind_address, port, heartbeat_port).await?;
    }

    Ok(())
}
