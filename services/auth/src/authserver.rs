use std::{fmt, net::Ipv4Addr, str};

use anyhow::{anyhow, Context, Result};
use async_std::{
    channel::{Receiver, Sender},
    net::{TcpListener, TcpStream},
    prelude::*,
    stream::StreamExt,
};
use bincode::Options;
use derivative::Derivative;
use derive_more::Display;
use game::{
    accounts::{AccountService, ConnectToken, ReconnectToken},
    realms::RealmList,
};
use tracing::{debug, error, info, instrument};

use crate::{
    protocol::{
        packets::{
            AuthCommand, ConnectChallenge, ConnectProof, ConnectProofResponse, ConnectRequest,
            Realm, RealmListResponse, ReconnectProof, ReplyPacket, ReturnCode, VERSION_CHALLENGE,
        },
        read_packet, Message,
    },
    wow_bincode::wow_bincode,
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
        token: ConnectToken,
    },

    #[display(fmt = "ReconnectChallenge")]
    ReconnectChallenge { token: ReconnectToken },

    // the server sends the challenge and gets a proof. this results
    // in either the authenticated or rejected states.
    /// The server has accepted the request.
    Realmlist,

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
                RequestState::Realmlist { .. } => AuthCommand::RealmList,
                RequestState::Rejected { command, .. } => *command,
                RequestState::Done => AuthCommand::RealmList,
                RequestState::ReconnectChallenge { .. } => AuthCommand::AuthReconnectProof,
            },
            reason,
        }
    }
}

/// Implements a WoW authentication server.
#[derive(Debug)]
pub struct AuthServer<T: AccountService + fmt::Debug, R: RealmList> {
    pub command_receiver: Receiver<Command>,
    pub reply_sender: Sender<ServerMessage>,
    pub accounts: T,
    pub realms: R,
}

impl<T: AccountService + fmt::Debug, R: RealmList> AuthServer<T, R> {
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
                    Ok(Message::ReconnectRequest(request)) => {
                        handle_reconnect_request(&request, &self.accounts, &state, stream).await?
                    }
                    Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                    Err(e) => return Err(e.into()),
                },
                RequestState::ConnectChallenge { ref token, .. } => {
                    match read_packet(&mut reader).await {
                        Ok(Message::ConnectProof(proof)) => {
                            handle_connect_proof(&proof, &self.accounts, token, &state, stream)
                                .await?
                        }
                        Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                        Err(e) => return Err(e.into()),
                    }
                }
                RequestState::ReconnectChallenge { ref token } => {
                    match read_packet(&mut reader).await {
                        Ok(Message::ReconnectProof(proof)) => {
                            handle_reconnect_proof(&proof, &self.accounts, token, &state, stream)
                                .await?
                        }
                        Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                        Err(e) => return Err(e.into()),
                    }
                }
                RequestState::Realmlist { .. } => match read_packet(&mut reader).await {
                    Ok(Message::RealmList(_)) => handle_realmlist(&self.realms, stream).await?,
                    Ok(p) => return Err(anyhow!("received message {} in state {}", p, state)),
                    Err(e) => return Err(e.into()),
                },
                RequestState::Rejected { command, reason } => {
                    let mut buffer = [0u8; 2];
                    wow_bincode().serialize_into(&mut buffer[..], &(command, reason))?;
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

#[instrument(skip(request, accounts, stream))]
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
                debug!("user connected with invalid username: {}", e);
                return Ok(RequestState::reject_from(&state, ReturnCode::Failed));
            }
        }
    };

    debug!("auth challenge for {}", username);

    let (state, response) = match accounts.initiate_login(username).await {
        Ok(token) => (RequestState::ConnectChallenge { token }, token.into()),
        Err(reason) => {
            return Ok(RequestState::reject_from(&state, reason.into()));
        }
    };

    let mut buffer = [0u8; 256];
    let packet = ReplyPacket::<ConnectChallenge>::new(response);

    // we need to use varint encoding here, otherwise the vector length will be encoded as a u64
    let len = wow_bincode()
        .with_varint_encoding()
        .serialized_size(&packet)? as usize;
    wow_bincode()
        .with_varint_encoding()
        .serialize_into(&mut buffer[..len], &packet)?;

    debug!(
        "writing {:?} ({} bytes) for {:?}",
        &buffer[..len],
        len,
        packet
    );
    stream.write(&buffer[..len]).await?;

    Ok(state)
}

#[instrument(skip(proof, accounts, token, state, stream))]
async fn handle_connect_proof(
    proof: &ConnectProof,
    accounts: &dyn AccountService,
    token: &ConnectToken,
    state: &RequestState,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    let (state, response) = match accounts
        .complete_login(token, &proof.user_public_key, &proof.user_proof)
        .await
    {
        Ok(server_proof) => (
            RequestState::Realmlist,
            ConnectProofResponse {
                error: 0,
                server_proof,
                account_flags: 0x00800000,
                survey_id: 0,
                login_flags: 0,
            },
        ),
        Err(status) => {
            return Ok(RequestState::reject_from(&state, status.into()));
        }
    };

    stream
        .write(&wow_bincode().serialize(&(AuthCommand::AuthLogonProof, response))?)
        .await?;

    Ok(state)
}

#[instrument(skip(request, accounts))]
async fn handle_reconnect_request(
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
    }

    let mut buffer = [0u8; 16];
    let username = {
        let username = &mut buffer[..request.identifier_length as usize];
        stream.read(username).await?;
        match str::from_utf8(username) {
            Ok(s) => s,
            Err(e) => {
                debug!("user connected with invalid username: {}", e);
                return Ok(RequestState::reject_from(&state, ReturnCode::Failed));
            }
        }
    };

    let token = match accounts.initiate_relogin(username).await {
        Ok(token) => token,
        Err(e) => return Ok(RequestState::reject_from(state, e.into())),
    };

    stream
        .write(&bincode::options().serialize(&(
            AuthCommand::AuthReconnectChallenge,
            ReturnCode::Success,
            token.reconnect_proof,
            VERSION_CHALLENGE,
        ))?)
        .await?;

    Ok(RequestState::ReconnectChallenge { token })
}

#[instrument(skip(proof, accounts, token, state, stream))]
async fn handle_reconnect_proof(
    proof: &ReconnectProof,
    accounts: &dyn AccountService,
    token: &ReconnectToken,
    state: &RequestState,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    let (state, response) = match accounts
        .complete_relogin(token, &proof.proof_data, &proof.client_proof)
        .await
    {
        Ok(_) => (
            RequestState::Realmlist,
            (AuthCommand::AuthReconnectProof, ReturnCode::Success, 0u16),
        ),
        Err(status) => {
            return Ok(RequestState::reject_from(&state, status.into()));
        }
    };

    debug!("user has reauthenticated");
    stream.write(&bincode::serialize(&response)?).await?;
    Ok(state)
}

#[instrument(skip(realms, stream))]
async fn handle_realmlist(realms: &dyn RealmList, stream: &mut TcpStream) -> Result<RequestState> {
    let realms = realms
        .realms()
        .await
        .iter()
        .map(|r| Realm::from_realm(&r, 0, false))
        .collect::<Vec<_>>();

    let resp = RealmListResponse::from_realms(&realms)?;
    let mut packet = Vec::with_capacity((resp.packet_size + 8).into());
    packet.append(&mut wow_bincode().serialize(&(AuthCommand::RealmList, resp))?);
    for realm in realms {
        packet.append(&mut wow_bincode().serialize(&realm)?);
    }
    packet.extend_from_slice(&[0x10, 0x0]);

    stream.write(&packet).await?;
    Ok(RequestState::Done)
}
