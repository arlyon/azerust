#![feature(arbitrary_enum_discriminant)]

use async_std::task;
use std::{net::Ipv4Addr, thread};

use anyhow::Result;
use authserver::AuthServer;
use conf::AuthServerConfig;
use human_panic::setup_panic;
use opt::Opt;
use structopt::StructOpt;

use ui::{Repl, Tui, UI};

mod authserver;
mod conf;
mod opt;
mod protocol;
mod state_machine;
mod ui;

fn main() -> Result<()> {
    setup_panic!();
    tracing_subscriber::fmt::init();

    let opts: Opt = Opt::from_args();

    match opts.command {
        opt::OptCommand::Exec(c) => match c {
            _ => {}
        },
        opt::OptCommand::Init => {
            let auth = AuthServerConfig {
                bind_address: "0.0.0.0".parse::<Ipv4Addr>().unwrap().into(),
                port: 3724,
                login_database: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            };
            auth.write(opts.config)?;
        }
        opt::OptCommand::Tui => start_server(opts, Some(Tui {}))?,
        opt::OptCommand::Repl => start_server(opts, Some(Repl {}))?,
        opt::OptCommand::Log => start_server::<Repl>(opts, None)?,
    };

    Ok(())
}

fn start_server<'a, U: 'static + UI + Send>(opts: Opt, ui: Option<U>) -> Result<()> {
    let config = AuthServerConfig::read(opts.config)?;

    let (command_sender, command_receiver) = async_std::channel::bounded::<authserver::Command>(10);
    let (reply_sender, reply_receiver) = async_std::channel::bounded::<authserver::Response>(10);

    let e1 = thread::spawn(move || match ui {
        Some(ui) => task::block_on(async {
            ui.start(&command_sender, &reply_receiver).await;
            command_sender.send(authserver::Command::ShutDown).await;
        }),
        None => task::block_on(async {
            loop {
                reply_receiver.recv().await;
            }
        }),
    });

    // let e2 = thread::spawn(move || {
    task::block_on(
        AuthServer(command_receiver, reply_sender).start(config.bind_address, config.port),
    );
    // });

    e1.join().unwrap();
    // e2.join().unwrap()

    Ok(())
}
