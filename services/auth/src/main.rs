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

use std::{net::Ipv4Addr, time::Duration};

use anyhow::{anyhow, Result};
use async_std::prelude::*;
use azerust_game::accounts::AccountService;
use azerust_mysql::{accounts::MySQLAccountService, realms::MySQLRealmList};
use azerust_tide::api;
use conf::AuthServerConfig;
use human_panic::setup_panic;
use sqlx::MySqlPool;
use structopt::StructOpt;
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

#[async_std::main]
async fn main() -> Result<()> {
    setup_panic!();
    tracing_subscriber::fmt::init();

    let opts: Opt = Opt::from_args();
    let config = AuthServerConfig::read(&opts.config).await;

    match opts.command {
        opt::OptCommand::Exec(c) => match c {
            opt::Command::Account {
                command:
                    AccountCommand::Create {
                        username,
                        password,
                        email,
                    },
            } => {
                let pool = MySqlPool::connect(&config?.login_database).await?;
                let accounts = MySQLAccountService::new(pool).await?;
                match accounts.create_account(&username, &password, &email).await {
                    Ok(id) => println!("created account {}", id),
                    Err(e) => eprintln!("failed to create account: {}", e),
                };
            }
            opt::Command::Shutdown => {}
        },
        opt::OptCommand::Init => {
            let auth = AuthServerConfig {
                bind_address: "0.0.0.0".parse::<Ipv4Addr>().expect("Valid IP"),
                port: 3724,
                api_port: None,
                login_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            };
            auth.write(&opts.config).await?;
        }
        opt::OptCommand::Run => start_server(&config?).await?,
    };

    Ok(())
}

async fn start_server(config: &AuthServerConfig) -> Result<()> {
    let pool = MySqlPool::connect(&config.login_database).await?;

    debug!("Loaded config {:?}", config);
    let accounts = MySQLAccountService::new(pool.clone()).await?;
    let realms = MySQLRealmList::new(pool.clone(), Duration::from_secs(60)).await?;

    let server = AuthServer {
        accounts: accounts.clone(),
        realms: realms.clone(),
    };

    let run = server.start(config.bind_address, config.port);

    if let Some(api_port) = config.api_port {
        let api = api(
            (config.bind_address.to_string(), api_port),
            accounts.clone(),
            realms.clone(),
        );

        run.try_join(async { api.await.map_err(|e| anyhow!(e)) })
            .await?;
    } else {
        run.await?;
    }

    Ok(())
}
