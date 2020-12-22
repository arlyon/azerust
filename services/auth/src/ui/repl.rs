use async_std::channel::Receiver;
use async_std::channel::Sender;

use anyhow::Result;
use authserver::Response;
use colored::*;
use rustyline::{error::ReadlineError, Editor};
use structopt::StructOpt;

use crate::authserver;
use crate::opt::{AccountCommand, Command};

use super::UI;
use async_trait::async_trait;

pub struct Repl;

#[async_trait]
impl UI for Repl {
    async fn start(&self, s: &Sender<authserver::Command>, r: &Receiver<Response>) -> Result<()> {
        loop {
            match r.recv().await {
                Ok(Response::Update(u)) => println!("{}", u),
                Ok(Response::Ready) => break,
                _ => {}
            }
        }

        let mut rl = Editor::<()>::new();
        let _ = rl.load_history(".auth.history");

        loop {
            let readline = rl.readline(">> ");

            match readline.map(|line| {
                rl.add_history_entry(line.as_str());

                let mut line = line.split(" ").collect::<Vec<&str>>();
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
                        .await
                        .unwrap();
                        print_output(&r).await;
                    }
                    c => println!("{:?}", c),
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

        s.send(authserver::Command::ShutDown).await.unwrap();
        print_output(&r).await;
        let _ = rl.save_history(".auth.history");

        Ok(())
    }
}

/// A simple read-eval-print loop.

async fn print_output(r: &Receiver<Response>) {
    while let Ok(x) = r.recv().await {
        match &x {
            Response::Update(x) => println!("{}", x),
            Response::Complete(x) => println!("{}", x.green()),
            Response::Error(x) => println!("{}", x.red()),
            _ => {}
        }

        match &x {
            Response::Complete(_) | Response::Error(_) => break,
            _ => {}
        }
    }
}
