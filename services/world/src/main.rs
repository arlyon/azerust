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

use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use anyhow::{anyhow, Context, Result};
use azerust_game::realms::RealmId;
use azerust_mysql_auth::{accounts::MySQLAccountService, realms::MySQLRealmList};
use azerust_mysql_characters::MySQLCharacterService;
use human_panic::setup_panic;
use sqlx::MySqlPool;
use structopt::StructOpt;
use tokio::{task::JoinHandle, try_join};
use tracing::debug;

use crate::{conf::WorldServerConfig, opt::Opt, worldserver::WorldServer};

mod client;
mod conf;
mod opt;
mod protocol;
mod world;
mod worldserver;
mod wow_bincode;

#[tokio::main]
async fn main() -> Result<()> {
    setup_panic!();

    let opts: Opt = Opt::from_args();
    let config = WorldServerConfig::read(&opts.config).await?;
    if let Some(port) = config.console_port {
        console_subscriber::ConsoleLayer::builder()
            .server_addr((config.bind_address, port))
            .init();
    }

    match opts.command {
        Some(opt::OptCommand::Init) => {
            let auth = WorldServerConfig {
                bind_address: "0.0.0.0".parse::<Ipv4Addr>().expect("Valid IP"),
                port: 3724,
                console_port: None,
                auth_server_address: "localhost:1234".to_string(),

                realm_id: RealmId(1),
                data_dir: 0,

                character_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
                auth_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
                world_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            };
            auth.write(&opts.config).await?;
        }
        None => start_server(&config).await?,
    };

    Ok(())
}

async fn flatten<T>(handle: JoinHandle<Result<T>>) -> Result<T> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(anyhow!("join failed: {}", err)),
    }
}

async fn start_server(config: &WorldServerConfig) -> Result<()> {
    let auth_pool = MySqlPool::connect(&config.auth_database)
        .await
        .context("could not start the database pool")?;

    let character_pool = MySqlPool::connect(&config.character_database)
        .await
        .context("could not start the database pool")?;

    debug!("Loaded config {:?}", config);

    let accounts = MySQLAccountService::new(auth_pool.clone());
    let realms = MySQLRealmList::new(auth_pool.clone(), Duration::from_secs(60));
    let characters = MySQLCharacterService::new(character_pool.clone());

    let server = Arc::new(WorldServer::new(
        config.realm_id,
        accounts,
        realms,
        characters,
        config.auth_server_address.clone(),
    ));

    try_join!(
        flatten(tokio::task::Builder::new().name("world::heartbeat").spawn({
            let cloned = server.clone();
            async move { cloned.auth_server_heartbeat().await }
        })),
        flatten(tokio::task::Builder::new().name("world::clients").spawn({
            let cloned = server.clone();
            async move { cloned.accept_clients().await }
        })),
        flatten(tokio::task::Builder::new().name("world::update").spawn({
            let cloned = server.clone();
            async move { cloned.update().await }
        })),
        flatten(tokio::task::Builder::new().name("world::packets").spawn({
            let cloned = server.clone();
            async move { cloned.world.handle_packets().await }
        })),
        flatten(tokio::task::Builder::new().name("world::timers").spawn({
            let cloned = server.clone();
            async move { cloned.world.timers().await }
        }))
    )?;

    Ok(())
}
