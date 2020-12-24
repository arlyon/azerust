use async_std::{
    channel::{Receiver, Sender},
    net::{TcpListener, TcpStream},
    prelude::*,
    stream::StreamExt,
};
use std::{convert::TryFrom, net::Ipv4Addr};

use anyhow::{anyhow, Context, Result};
use bincode::Options;
use derivative::Derivative;
use derive_more::Display;
use game::accounts::{Account, AccountService, BanStatus};
use thiserror::Error;
use tracing::{debug, error, info, instrument, trace};
use wow_srp::{Salt, Verifier, WowSRPServer};

use crate::protocol::{
    packets::{
        AuthCommand, ConnectChallenge, ConnectProof, ConnectProofResponse, ConnectRequest, Realm,
        RealmListResponse, ReconnectProof, ReplyPacket, ReplyPacket2, ReturnCode,
    },
    Message, MessageParseError,
};

/// Messages to the UI from the server
#[derive(PartialEq, Display, Debug)]
pub enum ServerMessage {
    Ready,
    Update(String),
    Complete(String),
    Error(String),
}

/// Messages to the server
#[derive(PartialEq, Eq)]
pub enum Command {
    NewAccount {
        /// The username of the new account
        username: String,
        /// The password to use
        password: String,
        /// The email address
        email: String,
    },
    ShutDown,
}

/// Models the various valid states of the server.
#[derive(Derivative)]
#[derivative(PartialEq, Debug)]
pub enum RequestState {
    /// The initial state, nothing has been provided.
    Start,

    /// The server receives a connect request and generates a challenge.
    ConnectChallenge {
        #[derivative(Debug = "ignore")]
        account: Account,
        server: WowSRPServer,
        #[derivative(Debug = "ignore")]
        response: ConnectChallenge,
    },

    // the server sends the challenge and gets a proof. this results
    // in either the authenticated or rejected states.
    /// The server has accepted the request.
    Authenticated {
        #[derivative(Debug = "ignore")]
        response: ConnectProofResponse,
    },

    /// The server has rejected the request.
    Rejected {
        stage: AuthCommand,
        status: ReturnCode,
    },

    /// We are done with the request.
    Done,
}

/// Implements a WoW authentication server.
#[derive(Debug)]
pub struct AuthServer<T: AccountService + std::fmt::Debug> {
    pub command_receiver: Receiver<Command>,
    pub reply_sender: Sender<ServerMessage>,
    pub accounts: T,
}

impl<T: AccountService + std::fmt::Debug> AuthServer<T> {
    /// Start the server, handling requests on the provided host and port.
    #[instrument(skip(self))]
    pub async fn start(&self, host: Ipv4Addr, port: u32) -> Result<()> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;

        info!("listening on {}", &addr);
        self.reply_sender
            .send(ServerMessage::Ready)
            .await
            .context("Couldn't send Authserver ready message")?;

        let mut connections = listener.incoming().filter_map(|s| s.ok());
        while let Some(mut stream) = connections.next().await {
            match self.connect_loop(&mut stream).await {
                Ok(_) => {}
                Err(e) => error!("error handling request: {}", e),
            }
        }

        Ok(())
    }

    #[instrument(skip(self, stream))]
    async fn connect_loop(&self, stream: &mut TcpStream) -> Result<()> {
        let mut reader = stream.clone();
        let mut state = RequestState::Start;

        loop {
            debug!("handling state {:?}", state);
            state = match state {
                RequestState::Start => match read_packet(&mut reader).await {
                    Ok(Message::ConnectRequest(c)) => {
                        let mut buffer = [0u8; 256];
                        let username = {
                            let username = &mut buffer[..c.identifier_length as usize];
                            reader.read(username).await?;
                            std::str::from_utf8(username).expect("valid")
                        };

                        let state = match handle_logon_request(c, username, &self.accounts).await {
                            Ok(s) => s,
                            Err(status) => {
                                state = RequestState::Rejected {
                                    stage: AuthCommand::ConnectRequest,
                                    status,
                                };
                                continue;
                            }
                        };

                        if let RequestState::ConnectChallenge { response, .. } = &state {
                            let packet = ReplyPacket::<ConnectChallenge>::new(response.clone());
                            let len = bincode::options().serialized_size(&packet)? as usize;
                            bincode::options().serialize_into(&mut buffer[..len], &packet)?;

                            debug!("packet: {:02X?}", &buffer[..len]);

                            stream.write(&buffer[..len]).await?;
                            stream.flush().await?;
                        }

                        state
                    }
                    Err(_e) => RequestState::Rejected {
                        stage: AuthCommand::ConnectRequest,
                        status: ReturnCode::Failed,
                    },
                    _ => return Err(anyhow!("invalid state")),
                },
                RequestState::ConnectChallenge {
                    account, server, ..
                } => match read_packet(&mut reader).await {
                    Ok(Message::AuthLogonProof(p)) => {
                        let state = match handle_logon_proof(p, server, &account).await {
                            Ok(s) => s,
                            Err(status) => {
                                state = RequestState::Rejected {
                                    stage: AuthCommand::AuthLogonProof,
                                    status,
                                };
                                continue;
                            }
                        };
                        if let RequestState::Authenticated { response } = &state {
                            let packet = ReplyPacket2 {
                                command: AuthCommand::AuthLogonProof,
                                message: *response,
                            };
                            let packet = bincode::serialize(&packet)?;

                            debug!("sending packet: {:?}", &packet);

                            stream.write(&packet).await?;
                            stream.flush().await?;
                        }

                        state
                    }
                    Err(e) => return Err(e.into()),
                    _ => return Err(anyhow!("invalid state")),
                },
                RequestState::Authenticated { .. } => match read_packet(&mut reader).await {
                    Ok(Message::RealmList(_r)) => {
                        let realms = vec![Realm {
                            realm_type: 0x01,
                            locked: false,
                            flags: 0x0,
                            name: "Hi Mum".into(),
                            socket: "51.178.64.97:8095".into(),
                            pop_level: 0f32,
                            character_count: 0,
                            timezone: 8,
                            realm_id: 1,
                        }];

                        let resp = RealmListResponse::from_realms(&realms)?;

                        let mut packet = bincode::options()
                            .with_fixint_encoding()
                            .serialize(&(AuthCommand::RealmList, resp))?;

                        for realm in realms {
                            debug!("sending realm {:?}", realm);
                            packet.append(
                                &mut bincode::options()
                                    .with_fixint_encoding()
                                    .with_null_terminated_str_encoding()
                                    .serialize(&realm)?,
                            );
                        }

                        packet.push(0x10);
                        packet.push(0x0);

                        stream.write(&packet).await?;
                        stream.flush().await?;

                        RequestState::Done
                    }
                    Err(e) => return Err(e.into()),
                    _ => return Err(anyhow!("invalid state")),
                },
                RequestState::Rejected { stage, status } => {
                    let mut buffer = [0u8; 2];
                    bincode::options().serialize_into(&mut buffer[..], &(stage, status))?;
                    debug!("sending {:?}", buffer);
                    stream.write(&buffer).await?;
                    stream.flush().await?;
                    RequestState::Done
                }
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
    packet.read(&mut buffer[..1]).await?;

    let command = AuthCommand::try_from(buffer[0]).expect("this should be valid");
    let command_len = match command {
        AuthCommand::ConnectRequest => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthLogonProof => std::mem::size_of::<ConnectProof>(),
        AuthCommand::AuthReconnectChallenge => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthReconnectProof => std::mem::size_of::<ReconnectProof>(),
        AuthCommand::RealmList => return Ok(Message::RealmList(Default::default())),
        _ => todo!(),
    };

    let read_length = packet.read(&mut buffer[..command_len]).await?;
    trace!(
        "read {:?} bytes into buffer {:02X?}",
        read_length,
        &buffer[..read_length]
    );

    if read_length != command_len {
        return Err(PacketHandleError::MessageLength(read_length, command_len));
    }

    match command {
        AuthCommand::ConnectRequest => bincode::deserialize(&buffer[..])
            .map(Message::ConnectRequest)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthLogonProof => bincode::deserialize(&buffer[..])
            .map(Message::AuthLogonProof)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthReconnectChallenge => bincode::deserialize(&buffer[..])
            .map(Message::AuthReconnectChallenge)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthReconnectProof => bincode::deserialize(&buffer[..])
            .map(Message::AuthReconnectProof)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        _ => Err(PacketHandleError::UnsupportedCommand(command)),
    }
}

#[instrument(skip(request, accounts))]
async fn handle_logon_request(
    request: ConnectRequest,
    username: &str,
    accounts: &dyn AccountService,
) -> Result<RequestState, ReturnCode> {
    debug!("auth challenge for {}", username);

    if request.build != 12340 {
        return Err(ReturnCode::VersionInvalid);
    };

    let account = accounts.get_account(username).await.ok();
    let account = match account {
        Some(Account {
            ban_status: Some(status),
            username,
            ..
        }) => {
            debug!("banned user {} attempted to log in", username);
            return match status {
                BanStatus::Temporary => Err(ReturnCode::Suspended),
                BanStatus::Permanent => Err(ReturnCode::Banned),
            };
        }
        Some(x) => x,
        None => {
            return Err(ReturnCode::UnknownAccount);
        }
    };

    debug!("got user {:?}", account);

    let salt = Salt(account.salt);
    let verifier = Verifier(account.verifier);
    let server = WowSRPServer::new(&account.username, salt, verifier);

    Ok(RequestState::ConnectChallenge {
        response: ConnectChallenge {
            server,
            security_flags: 0,
        },
        server,
        account,
    })
}

#[derive(Error, Debug)]
pub enum PacketHandleError {
    #[error("could not parse message: {0}")]
    MessageParse(#[from] MessageParseError),

    #[error("received {0}, expected {1}")]
    MessageLength(usize, usize),

    #[error("error while reading packet: {0}")]
    IoRead(#[from] async_std::io::Error),

    #[error("command is not supported: {0}")]
    UnsupportedCommand(AuthCommand),
}

async fn handle_logon_proof(
    p: ConnectProof,
    server: WowSRPServer,
    _account: &Account,
) -> Result<RequestState, ReturnCode> {
    let session_key = match server.verify_challenge_response(&p.user_public_key, &p.user_proof) {
        Some(k) => k,
        None => {
            debug!("failed password");
            return Err(ReturnCode::IncorrectPassword);
        }
    };

    let server_proof = server.get_server_proof(&p.user_public_key, &p.user_proof, &session_key);

    println!("server proof: {:?}", server_proof);

    let response = ConnectProofResponse {
        error: 0,
        server_proof,
        account_flags: 0x00800000,
        survey_id: 0,
        login_flags: 0,
    };

    Ok(RequestState::Authenticated { response })
}
