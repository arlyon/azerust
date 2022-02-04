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
use conf::AuthServerConfig;
use human_panic::setup_panic;
use sqlx::MySqlPool;
use structopt::StructOpt;
use tokio::try_join;
use tracing::debug;

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
    tracing_subscriber::fmt::init();

    let opts = Opt::from_args();
    let config = AuthServerConfig::read(&opts.config).await;

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
                let pool = MySqlPool::connect(&config?.auth_database).await?;
                let accounts = MySQLAccountService::new(pool);
                match accounts.create_account(&username, &password, &email).await {
                    Ok(id) => println!("created account {}", id),
                    Err(e) => eprintln!("failed to create account: {}", e),
                };
            }
        },
        Some(opt::OptCommand::Init) => {
            let auth = AuthServerConfig {
                bind_address: "0.0.0.0".parse::<Ipv4Addr>().expect("Valid IP"),
                port: 3724,
                api_port: None,
                auth_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            };
            auth.write(&opts.config).await?;
        }
        None => start_server(&config?).await?,
    };

    Ok(())
}

async fn start_server(config: &AuthServerConfig) -> Result<()> {
    let pool = MySqlPool::connect(&config.auth_database).await?;

    debug!("Loaded config {:?}", config);
    let accounts = MySQLAccountService::new(pool.clone());
    let realms = MySQLRealmList::new(pool.clone(), Duration::from_secs(10));

    let server = AuthServer::new(accounts.clone(), realms.clone());
    let run = server.start(config.bind_address, config.port);

    if let Some(api_port) = config.api_port {
        let addr = SocketAddr::new(config.bind_address.into(), api_port);
        let api = api(&addr, accounts.clone(), realms.clone());

        try_join!(run, async { api.await.map_err(|e| anyhow!("bad api")) })?;
    } else {
        run.await?;
    }

    Ok(())
}
