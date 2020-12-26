#![feature(arbitrary_enum_discriminant)]
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

use anyhow::Result;
use human_panic::setup_panic;
use structopt::StructOpt;

use crate::{
    authserver::AuthServer,
    opt::{AccountCommand, Opt},
    ui::{Repl, Tui, UI},
};
use conf::AuthServerConfig;
use game::accounts::AccountService;
use mysql::{accounts::MySQLAccountService, realms::MySQLRealmList};

mod authserver;
mod conf;
mod opt;
mod protocol;
mod ui;

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
                let accounts = MySQLAccountService::new(&config?.login_database).await?;
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
                login_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            };
            auth.write(&opts.config).await?;
        }
        opt::OptCommand::Tui => start_server(opts, Some(Tui {}), &config?).await?,
        opt::OptCommand::Repl => start_server(opts, Some(Repl {}), &config?).await?,
        opt::OptCommand::Log => start_server::<Repl>(opts, None, &config?).await?,
    };

    Ok(())
}

async fn start_server<U: 'static + UI + Send>(
    _opts: Opt,
    _ui: Option<U>,
    config: &AuthServerConfig,
) -> Result<()> {
    let (_command_sender, command_receiver) =
        async_std::channel::bounded::<authserver::Command>(10);
    let (reply_sender, _reply_receiver) =
        async_std::channel::bounded::<authserver::ServerMessage>(10);

    // let e1 = thread::spawn(move || match ui {
    //     Some(ui) => task::block_on(async {
    //         ui.start(&command_sender, &reply_receiver).await.unwrap();
    //         command_sender
    //             .send(authserver::Command::ShutDown)
    //             .await
    //             .unwrap();
    //     }),
    //     None => task::block_on(async {
    //         loop {
    //             reply_receiver.recv().await;
    //         }
    //     }),
    // });

    let accounts = MySQLAccountService::new("mysql://localhost:49153/auth").await?;
    let realms =
        MySQLRealmList::new("mysql://localhost:49153/auth", Duration::from_secs(60)).await?;

    AuthServer {
        command_receiver,
        reply_sender,
        accounts,
        realms,
    }
    .start(config.bind_address, config.port)
    .await
}
