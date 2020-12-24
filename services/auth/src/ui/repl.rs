use async_std::channel::{Receiver, Sender};

use anyhow::Result;
use authserver::ServerMessage;
use colored::*;
use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

use crate::{
    authserver,
    opt::{AccountCommand, Command},
};

use super::UI;
use async_trait::async_trait;

pub struct Repl;

#[async_trait]
impl UI for Repl {
    async fn start(
        &self,
        s: &Sender<authserver::Command>,
        r: &Receiver<ServerMessage>,
    ) -> Result<()> {
        loop {
            match r.recv().await {
                Ok(ServerMessage::Update(u)) => println!("{}", u),
                Ok(ServerMessage::Ready) => break,
                _ => {}
            }
        }

        let mut rl = Editor::<()>::new();
        let _ = rl.load_history(".auth.history");

        loop {
            let readline = rl.readline(">> ");

            match readline.map(|line| {
                rl.add_history_entry(line.as_str());

                let mut line = line.split(' ').collect::<Vec<&str>>();
                line.insert(0, "");

                Command::from_iter_safe(line)
            }) {
                Ok(Ok(c)) => match c {
                    Command::Shutdown => break,
                    Command::Account {
                        command:
                            AccountCommand::Create {
                                email,
                                password,
                                username,
                            },
                    } => {
                        s.send(authserver::Command::NewAccount {
                            email,
                            password,
                            username,
                        })
                        .await?;
                        print_output(&r).await;
                    }
                },
                Ok(Err(e)) => {
                    println!("{}", e.message);
                }
                Err(ReadlineError::Interrupted) => {
                    break;
                }
                Err(ReadlineError::Eof) => {
                    break;
                }
                Err(err) => {
                    println!("Could not read input: {:?}", err);
                }
            }
        }

        s.send(authserver::Command::ShutDown).await?;
        print_output(&r).await;
        let _ = rl.save_history(".auth.history");

        Ok(())
    }
}

/// A simple read-eval-print loop.

async fn print_output(r: &Receiver<ServerMessage>) {
    while let Ok(x) = r.recv().await {
        match &x {
            ServerMessage::Update(x) => println!("{}", x),
            ServerMessage::Complete(x) => println!("{}", x.green()),
            ServerMessage::Error(x) => println!("{}", x.red()),
            _ => {}
        }

        match &x {
            ServerMessage::Complete(_) | ServerMessage::Error(_) => break,
            _ => {}
        }
    }
}
