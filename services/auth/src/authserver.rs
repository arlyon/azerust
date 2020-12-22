use async_std::{
    channel::{Receiver, Sender},
    io::BufReader,
    net::{TcpListener, TcpStream},
    prelude::*,
    stream::StreamExt,
};
use bincode::Options;
use game::accounts::{Account, AccountService, BanStatus};
use num_bigint::BigUint;
use rand::Rng;
use srp::server::{SrpServer, UserRecord};
use std::{
    convert::{TryFrom, TryInto},
    net::Ipv4Addr,
    time::Duration,
};

use anyhow::{Context, Result};
use derive_more::Display;
use thiserror::Error;
use tracing::{event, instrument, Level};

use crate::{
    protocol::{
        packets::{
            AuthCommand, AuthLogonProof, AuthLogonProofResponse, AuthReconnectProof,
            ConnectChallenge, ConnectRequest, LogonProof, ReplyPacket,
        },
        Message, MessageParseError,
    },
    state_machine::{InitState, Machine},
};

#[derive(PartialEq, Eq)]
pub enum Command {
    NewAccount {
        /// The username of the new account
        username: String,
        /// The password to use
        password: String,
        /// The email address
        email: Option<String>,
    },
    ShutDown,
}

#[derive(PartialEq, Display)]
pub enum Response {
    Ready,
    Update(String),
    Complete(String),
    Error(String),
}

pub enum ResponseData {
    AuthLogonChallenge(ConnectChallenge),
    AuthLogonProof(AuthLogonProofResponse),
}

#[derive(Debug)]
pub struct AuthServer(pub Receiver<Command>, pub Sender<Response>);

impl AuthServer {
    #[instrument(skip(self))]
    pub async fn start(&self, host: Ipv4Addr, port: u32) -> Result<()> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;

        event!(Level::DEBUG, "listening on {}", &addr);
        self.1
            .send(Response::Ready)
            .await
            .context("Couldn't send Authserver ready message")?;

        let mut connections = listener.incoming().filter_map(|s| s.ok());
        while let Some(mut stream) = connections.next().await {
            self.connect_loop(&mut stream).await;
        }

        Ok(())
    }

    /// A connection may contain multiple packets
    #[instrument(skip(self, stream))]
    async fn connect_loop(&self, stream: &mut TcpStream) -> Result<()> {
        let mut reader = stream.clone();

        let accounts = mysql::accounts::AccountService::new("mysql://localhost:49156/auth")
            .await
            .unwrap();

        let state = Machine::<InitState>::new();

        let message = read_packet(&mut reader).await?;
        match message {
            Some(Message::AuthLogonChallenge(c)) => {
                let mut username = [0u8; 16];
                let username = &mut username[..c.I_len as usize];
                reader.read(username).await?;
                let username = std::str::from_utf8(username).expect("valid");

                let state = match state.submit_request(c, username, &accounts).await {
                    Ok(l) => l,
                    Err(c) => todo!(),
                };

                let packet = ReplyPacket::new(state.get_challenge());
                let packet = bincode::DefaultOptions::new()
                    .with_varint_encoding()
                    .serialize(&packet)
                    .unwrap();
                stream.write(&packet).await?;
                stream.flush().await?;

                let message = read_packet(&mut reader).await?;
                let proof = match message {
                    Some(Message::AuthLogonProof(p)) => p,
                    _ => todo!(),
                };

                let state = match state.give_proof(proof) {
                    Ok(p) => p,
                    Err(c) => todo!(),
                };
                let realmlist = state.get_realmlist();
                let state = state.close();
            }
            Some(Message::AuthReconnectChallenge(c)) => {
                todo!();
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[instrument(skip(packet))]
pub async fn read_packet<R: async_std::io::Read + std::fmt::Debug + Unpin>(
    packet: &mut R,
) -> Result<Option<Message>, PacketHandleError> {
    let mut buffer = [0u8; 128];

    event!(Level::DEBUG, "getting command");
    if packet.read(&mut buffer[..1]).await.unwrap() == 0 {
        return Ok(None);
    }

    let command = AuthCommand::try_from(buffer[0]).expect("this should be valid");
    event!(Level::DEBUG, "received command {:?}", command);

    let command_len = match command {
        AuthCommand::ConnectRequest => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthLogonProof => std::mem::size_of::<AuthLogonProof>(),
        AuthCommand::AuthReconnectChallenge => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthReconnectProof => std::mem::size_of::<AuthReconnectProof>(),
        AuthCommand::RealmList => return Ok(Some(Message::RealmList)),
        _ => todo!(),
    };

    assert!(packet.read(&mut buffer[..command_len]).await.unwrap() == command_len);
    event!(Level::DEBUG, "read buffer {:?}", &buffer[..command_len]);

    match command {
        AuthCommand::ConnectRequest => bincode::deserialize(&buffer[..])
            .map(|d| Message::AuthLogonChallenge(d))
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthLogonProof => bincode::deserialize(&buffer[..])
            .map(|d| Message::AuthLogonProof(d))
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthReconnectChallenge => bincode::deserialize(&buffer[..])
            .map(|d| Message::AuthReconnectChallenge(d))
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthReconnectProof => bincode::deserialize(&buffer[..])
            .map(|d| Message::AuthReconnectProof(d))
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        _ => todo!(),
    }
    .map(|m| Some(m))
    .map_err(Into::into)
}

#[instrument(skip(request))]
async fn handle_logon_challenge(request: ConnectRequest, username: &[u8]) -> ConnectChallenge {}

#[derive(Error, Debug)]
pub enum PacketHandleError {
    #[error("could not parse message: {0}")]
    MessageParse(#[from] MessageParseError),

    #[error("received {0}, expected {1}")]
    StatusError(u32, u32),
}

async fn handle_logon_proof(p: AuthLogonProof) -> AuthLogonProofResponse {
    // get srp6 Server
    // server::verify(client_M);
}
