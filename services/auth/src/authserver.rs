use async_std::{
    channel::{Receiver, Sender},
    net::{TcpListener, TcpStream},
    prelude::*,
    stream::StreamExt,
};
use std::{net::Ipv4Addr, str};

use anyhow::{anyhow, Context, Result};
use bincode::Options;
use derivative::Derivative;
use derive_more::Display;
use game::{
    accounts::{AccountService, LoginHandler},
    realms::RealmList,
};
use tracing::{debug, error, info, instrument};

use crate::protocol::{
    packets::{
        AuthCommand, ConnectChallenge, ConnectProof, ConnectProofResponse, ConnectRequest, Realm,
        RealmListResponse, ReplyPacket, ReplyPacket2, ReturnCode,
    },
    read_packet, Message,
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
#[derive(Derivative, Display)]
#[derivative(PartialEq, Debug)]
pub enum RequestState {
    /// The initial state, nothing has been provided.
    Start,

    /// The server receives a connect request and generates a challenge.
    #[display(fmt = "ConnectChallenge")]
    ConnectChallenge {
        #[derivative(Debug = "ignore")]
        login_handler: LoginHandler,
        #[derivative(Debug = "ignore")]
        response: ConnectChallenge,
    },

    // the server sends the challenge and gets a proof. this results
    // in either the authenticated or rejected states.
    /// The server has accepted the request.
    #[display(fmt = "Authenticated")]
    Authenticated {
        #[derivative(Debug = "ignore")]
        response: ConnectProofResponse,
    },

    /// The server has rejected the request.
    #[display(fmt = "Rejected")]
    Rejected {
        command: AuthCommand,
        reason: ReturnCode,
    },

    /// We are done with the request.
    Done,
}

impl RequestState {
    fn reject_from(state: &Self, reason: ReturnCode) -> Self {
        Self::Rejected {
            command: match state {
                RequestState::Start => AuthCommand::ConnectRequest,
                RequestState::ConnectChallenge { .. } => AuthCommand::AuthLogonProof,
                RequestState::Authenticated { .. } => AuthCommand::RealmList,
                RequestState::Rejected { command, .. } => *command,
                RequestState::Done => AuthCommand::RealmList,
            },
            reason,
        }
    }
}

/// Implements a WoW authentication server.
#[derive(Debug)]
pub struct AuthServer<T: AccountService + std::fmt::Debug, R: RealmList> {
    pub command_receiver: Receiver<Command>,
    pub reply_sender: Sender<ServerMessage>,
    pub accounts: T,
    pub realms: R,
}

impl<T: AccountService + std::fmt::Debug, R: RealmList> AuthServer<T, R> {
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
                    Ok(Message::ConnectRequest(request)) => {
                        handle_connect_request(&request, &self.accounts, &state, stream).await?
                    }
                    Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                    Err(e) => return Err(e.into()),
                },
                RequestState::ConnectChallenge {
                    ref login_handler, ..
                } => match read_packet(&mut reader).await {
                    Ok(Message::ConnectProof(proof)) => {
                        handle_connect_proof(&proof, login_handler, &state, stream).await?
                    }
                    Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                    Err(e) => return Err(e.into()),
                },
                RequestState::Authenticated { .. } => match read_packet(&mut reader).await {
                    Ok(Message::RealmList(_)) => handle_realmlist(&self.realms, stream).await?,
                    Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                    Err(e) => return Err(e.into()),
                },
                RequestState::Rejected { command, reason } => {
                    let mut buffer = [0u8; 2];
                    bincode::options().serialize_into(&mut buffer[..], &(command, reason))?;
                    info!("rejecting {} due to {}", command, reason);
                    stream.write(&buffer).await?;
                    RequestState::Done
                }
                RequestState::Done => break,
            }
        }

        Ok(())
    }
}

#[instrument(skip(request, accounts))]
async fn handle_connect_request(
    request: &ConnectRequest,
    accounts: &dyn AccountService,
    state: &RequestState,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    if request.build != 12340 {
        return Ok(RequestState::reject_from(
            &state,
            ReturnCode::VersionInvalid,
        ));
    };

    let mut buffer = [0u8; 16];
    let username = {
        let username = &mut buffer[..request.identifier_length as usize];
        stream.read(username).await?;
        match str::from_utf8(username) {
            Ok(s) => s,
            Err(e) => {
                error!("user connected with invalid username: {}", e);
                return Ok(RequestState::reject_from(&state, ReturnCode::Failed));
            }
        }
    };

    debug!("auth challenge for {}", username);

    let state = match accounts
        .initiate_login(username)
        .await
        .map(|login_handler| RequestState::ConnectChallenge {
            response: ConnectChallenge::from_login_handler(&login_handler),
            login_handler,
        }) {
        Ok(s) => s,
        Err(reason) => {
            return Ok(RequestState::reject_from(&state, reason.into()));
        }
    };

    if let RequestState::ConnectChallenge { response, .. } = &state {
        let mut buffer = [0u8; 256];
        let packet = ReplyPacket::<ConnectChallenge>::new(response.clone());
        let len = bincode::options().serialized_size(&packet)? as usize;
        bincode::options().serialize_into(&mut buffer[..len], &packet)?;
        stream.write(&buffer[..len]).await?;
    }

    Ok(state)
}

#[instrument(skip(proof, login_handler))]
async fn handle_connect_proof(
    proof: &ConnectProof,
    login_handler: &LoginHandler,
    state: &RequestState,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    let state = match login_handler
        .login(&proof.user_public_key, &proof.user_proof)
        .await
        .map(|server_proof| RequestState::Authenticated {
            response: ConnectProofResponse {
                error: 0,
                server_proof,
                account_flags: 0x00800000,
                survey_id: 0,
                login_flags: 0,
            },
        }) {
        Ok(s) => s,
        Err(status) => {
            return Ok(RequestState::reject_from(&state, status.into()));
        }
    };
    if let RequestState::Authenticated { response } = &state {
        stream
            .write(&bincode::serialize(&ReplyPacket2 {
                command: AuthCommand::AuthLogonProof,
                message: *response,
            })?)
            .await?;
    }
    Ok(state)
}

async fn handle_realmlist(realms: &dyn RealmList, stream: &mut TcpStream) -> Result<RequestState> {
    let realms = realms
        .realms()
        .await
        .iter()
        .map(|r| Realm::from_realm(&r, 0, false))
        .collect::<Vec<_>>();

    let resp = RealmListResponse::from_realms(&realms)?;
    let mut packet = Vec::with_capacity((resp.packet_size + 8).into());
    packet.append(&mut bincode::serialize(&(AuthCommand::RealmList, resp))?);
    for realm in realms {
        packet.append(
            &mut bincode::options()
                .with_fixint_encoding()
                .with_null_terminated_str_encoding()
                .serialize(&realm)?,
        );
    }
    packet.extend_from_slice(&[0x10, 0x0]);

    stream.write(&packet).await?;
    Ok(RequestState::Done)
}
