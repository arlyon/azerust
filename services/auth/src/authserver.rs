use async_std::{
    channel::{Receiver, Sender},
    net::{TcpListener, TcpStream},
    prelude::*,
    stream::StreamExt,
};
use bincode::Options;
use game::accounts::{Account, AccountService};
use num_bigint::BigUint;
use rand::Rng;
use sha1::Sha1;
use srp::server::{SrpServer, UserRecord};
use std::{
    convert::{TryFrom, TryInto},
    net::Ipv4Addr,
};

use anyhow::{anyhow, Context, Result};
use derivative::Derivative;
use derive_more::Display;
use thiserror::Error;
use tracing::{event, instrument, Level};

use crate::protocol::{
    packets::{
        AuthCommand, ConnectChallenge, ConnectProof, ConnectProofResponse, ConnectRequest, Realm,
        RealmListResponse, ReconnectProof, ReplyPacket,
    },
    Message, MessageParseError,
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

#[derive(Derivative)]
#[derivative(PartialEq)]
pub enum RequestState {
    /// The initial state, nothing has been provided.
    Start,

    /// The server receives a connect request and generates a challenge.
    ConnectChallenge {
        account: Account,

        #[derivative(PartialEq = "ignore")]
        server: SrpServer<Sha1>,

        response: ConnectChallenge,
    },

    // the server sends the challenge and gets a proof. this results
    // in either the authenticated or rejected states.
    /// The server has accepted the request.
    Authenticated {
        response: ConnectProofResponse,
    },

    /// The server has rejected the request.
    Rejected,

    Done,
}

#[derive(PartialEq, Display)]
pub enum Response {
    Ready,
    Update(String),
    Complete(String),
    Error(String),
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
            let _ = self.connect_loop(&mut stream).await;
        }

        Ok(())
    }

    /// A connection may contain multiple packets
    #[instrument(skip(self, stream))]
    async fn connect_loop(&self, stream: &mut TcpStream) -> Result<()> {
        let mut reader = stream.clone();
        let mut state = RequestState::Start;
        let accounts = mysql::accounts::AccountService::new("mysql://localhost:49156/auth")
            .await
            .unwrap();

        loop {
            state = match state {
                RequestState::Start => {
                    let message = read_packet(&mut reader).await?;
                    match message {
                        Message::ConnectRequest(c) => {
                            let mut buffer = [0u8; 256];
                            let username = {
                                let username = &mut buffer[..c.identifier_length as usize];
                                reader.read(username).await?;
                                std::str::from_utf8(username).expect("valid")
                            };

                            let state = handle_logon_request(c, username, &accounts).await?;
                            if let RequestState::ConnectChallenge { response, .. } = &state {
                                let packet = ReplyPacket::new(response.clone());
                                let len = bincode::DefaultOptions::new()
                                    .with_varint_encoding()
                                    .serialized_size(&packet)
                                    .unwrap();
                                bincode::DefaultOptions::new()
                                    .with_varint_encoding()
                                    .serialize_into(&mut buffer[..], &packet)
                                    .unwrap();
                                stream.write(&buffer[..len as usize]).await?;
                                stream.flush().await?;
                            }

                            state
                        }
                        _ => return Err(anyhow!("invalid state")),
                    }
                }
                RequestState::ConnectChallenge {
                    account, server, ..
                } => {
                    let message = read_packet(&mut reader).await?;
                    match message {
                        Message::AuthLogonProof(p) => {
                            let state = handle_logon_proof(p, server, account).await?;
                            if let RequestState::Authenticated { response } = &state {
                                let packet = ReplyPacket::new(response.clone());
                                let packet = bincode::DefaultOptions::new()
                                    .with_varint_encoding()
                                    .serialize(&packet)
                                    .unwrap();
                                stream.write(&packet).await?;
                                stream.flush().await?;
                            }

                            state
                        }
                        _ => return Err(anyhow!("invalid state")),
                    }
                }
                RequestState::Authenticated { response } => {
                    let message = read_packet(&mut reader).await?;
                    match message {
                        Message::RealmList(_r) => {
                            let resp = RealmListResponse {
                                packet_size: 200,
                                data: [0; 4],
                                realm_count: 1,
                            };

                            let realms = [Realm {
                                realm_type: 0x1,
                                status: 0x0,
                                color: 0x0,
                                name: "Blackrock".to_string(),
                                socket: "localhost:8080".to_string(),
                                pop_level: 0,
                                character_count: 0,
                                timezone: 8,
                            }];

                            let packet = bincode::DefaultOptions::new()
                                .with_varint_encoding()
                                .serialize(&resp)
                                .unwrap();
                            stream.write(&packet).await?;
                            stream.flush().await?;

                            RequestState::Done
                        }
                        _ => return Err(anyhow!("invalid state")),
                    }
                }
                RequestState::Rejected => break,
                RequestState::Done => break,
            }
        }

        Ok(())
    }
}

#[instrument(skip(packet))]
pub async fn read_packet<R: async_std::io::Read + std::fmt::Debug + Unpin>(
    packet: &mut R,
) -> Result<Message, PacketHandleError> {
    let mut buffer = [0u8; 128];

    event!(Level::DEBUG, "getting command");
    packet.read(&mut buffer[..1]).await.unwrap();

    let command = AuthCommand::try_from(buffer[0]).expect("this should be valid");
    event!(Level::DEBUG, "received command {:?}", command);

    let command_len = match command {
        AuthCommand::ConnectRequest => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthLogonProof => std::mem::size_of::<ConnectProof>(),
        AuthCommand::AuthReconnectChallenge => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthReconnectProof => std::mem::size_of::<ReconnectProof>(),
        AuthCommand::RealmList => return Ok(Message::RealmList(Default::default())),
        _ => todo!(),
    };

    assert!(packet.read(&mut buffer[..command_len]).await.unwrap() == command_len);
    event!(Level::DEBUG, "read buffer {:?}", &buffer[..command_len]);

    match command {
        AuthCommand::ConnectRequest => bincode::deserialize(&buffer[..])
            .map(|d| Message::ConnectRequest(d))
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
}

#[instrument(skip(request, accounts))]
async fn handle_logon_request(
    request: ConnectRequest,
    username: &str,
    accounts: &dyn AccountService,
) -> Result<RequestState> {
    event!(Level::DEBUG, "auth challenge for {}", username);

    if request.build != 12340 {
        // return Err(ConnectChallenge::VersionInvalid);
        todo!()
    };

    let account = accounts.get_account(username).await.ok();
    let account = match account {
        Some(Account {
            ban_status: Some(status),
            username,
            ..
        }) => {
            event!(Level::DEBUG, "banned user {} attempted to log in", username);
            todo!()
            // return match status {
            //     BanStatus::Temporary => Err(ConnectChallenge::Suspended),
            //     BanStatus::Permanent => Err(ConnectChallenge::Banned),
            // };
        }
        Some(x) => x,
        None => {
            todo!()
            // return Err(ConnectChallenge::UnknownAccount);
        }
    };

    event!(Level::DEBUG, "got user {:?}", account);

    let group = srp::types::SrpGroup {
        n: BigUint::from_bytes_be(&[
            0x89, 0x4B, 0x64, 0x5E, 0x89, 0xE1, 0x53, 0x5B, 0xBD, 0xAD, 0x5B, 0x8B, 0x29, 0x06,
            0x50, 0x53, 0x08, 0x01, 0xB1, 0x8E, 0xBF, 0xBF, 0x5E, 0x8F, 0xAB, 0x3C, 0x82, 0x87,
            0x2A, 0x3E, 0x9B, 0xB7,
        ]),
        g: BigUint::from_bytes_be(&[7]),
    };
    let user_record = UserRecord {
        salt: &account.salt,
        username: account.username.as_bytes(),
        verifier: &account.verifier,
    };

    let mut b = [0u8; 64];
    let fst: [u8; 32] = rand::thread_rng().gen();
    let snd: [u8; 32] = rand::thread_rng().gen();
    b[..32].copy_from_slice(&fst);
    b[32..].copy_from_slice(&snd);

    // generate dummy a value, so we can get the B
    let a_dummy: [u8; 32] = rand::thread_rng().gen();

    let srp_1 = SrpServer::new(&user_record, &a_dummy, &b, &group).expect("works");
    let srp_2 = SrpServer::new(&user_record, &a_dummy, &b, &group).expect("works");

    let response = ConnectChallenge {
        srp: srp_1,
        group: group,
        salt: user_record.salt.try_into().unwrap(),
        security_flags: 0,
    };

    Ok(RequestState::ConnectChallenge {
        response,
        server: srp_2,
        account: account,
    })
}

#[derive(Error, Debug)]
pub enum PacketHandleError {
    #[error("could not parse message: {0}")]
    MessageParse(#[from] MessageParseError),

    #[error("received {0}, expected {1}")]
    StatusError(u32, u32),
}

async fn handle_logon_proof(
    p: ConnectProof,
    server: SrpServer<Sha1>,
    account: Account,
) -> Result<RequestState> {
    let server_proof = match server.verify(&p.user_proof) {
        Ok(k) => k,
        Err(e) => {
            todo!("auth logon proof, unkown account")
        }
    };

    let response = ConnectProofResponse {
        error: 0,
        server_proof: server_proof.as_slice().try_into().unwrap(),
        account_flags: 0x0,
        survey_id: 0,
        login_flags: 0,
    };

    Ok(RequestState::Authenticated { response })
}

pub fn verify_version() -> bool {
    true
}
