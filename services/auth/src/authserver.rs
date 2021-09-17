use std::{
    collections::HashMap,
    fmt,
    net::Ipv4Addr,
    str,
    time::{self, Instant},
};

use anyhow::{anyhow, Result};
use async_std::{
    net::{TcpListener, TcpStream},
    prelude::*,
    stream::{self, StreamExt},
    sync::RwLock,
};
use azerust_game::{
    accounts::{AccountService, ConnectToken, ReconnectToken},
    realms::{RealmFlags, RealmList},
};
use bincode::Options;
use derivative::Derivative;
use derive_more::Display;
use tracing::{debug, error, info, instrument, trace};

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

/// Implements a WoW authentication server.
#[derive(Debug)]
pub struct AuthServer<T: AccountService + fmt::Debug, R: RealmList> {
    accounts: T,
    realms: R,
    heartbeat: RwLock<HashMap<u8, Instant>>,
}

impl<T: AccountService + fmt::Debug, R: RealmList> AuthServer<T, R> {
    pub fn new(accounts: T, realms: R) -> Self {
        Self {
            accounts,
            realms,
            heartbeat: RwLock::new(HashMap::new()),
        }
    }

    /// Start the server, handling requests on the provided host and port.
    pub async fn start(&self, host: Ipv4Addr, port: u16) -> Result<()> {
        self.authentication(host, port)
            .try_join(self.world_server_heartbeat(host))
            .try_join(self.realmlist_updater())
            .await
            .map(|_| ())
    }

    #[instrument(skip(self, host))]
    async fn world_server_heartbeat(&self, host: Ipv4Addr) -> Result<()> {
        // todo(arlyon): change the world server listen port
        let socket = async_std::net::UdpSocket::bind((host, 1234)).await?;

        let mut buffer = [0u8; 6];
        loop {
            if let Err(_) = socket.recv(&mut buffer).await {
                debug!("received larger packet than expected");
                continue;
            };
            match wow_bincode().deserialize(&buffer) {
                Ok((0u8, realm_id, realm_pop)) => {
                    self.heartbeat
                        .write()
                        .await
                        .insert(realm_id, Instant::now());
                    trace!(
                        "got heartbeat for {} with realm pop {}",
                        realm_id,
                        realm_pop
                    )
                }
                Ok((_, _, 0u32)) | _ => debug!("received bad buffer: {:02X?}", &buffer),
            }
        }
    }

    /// updates the realmlist based on recently received heartbeats
    #[instrument(skip(self))]
    async fn realmlist_updater(&self) -> Result<()> {
        let instant = stream::from_fn(|| Some(Instant::now()));
        let mut interval = stream::interval(time::Duration::from_secs(5)).zip(instant);
        while let Some((_, now)) = interval.next().await {
            let data = {
                let mut write = self.heartbeat.write().await;
                let mut data = Vec::with_capacity(write.len());
                data.extend(
                    write
                        .drain_filter(|_, v| now.duration_since(*v).as_secs() > 15)
                        .map(|(k, _)| (k, RealmFlags::Offline)),
                );
                data.extend(write.keys().map(|&k| (k, RealmFlags::Recommended)));
                data
            };
            trace!("updating realm populations: {:?}", data);
            self.realms.update_status(data).await;
        }
        Ok(())
    }

    #[instrument(skip(self, host, port))]
    async fn authentication(&self, host: Ipv4Addr, port: u16) -> Result<()> {
        let addr = (host, port);
        let listener = TcpListener::bind(&addr).await?;

        info!("listening on {:?}", &addr);

        let mut connections = listener.incoming().filter_map(|s| s.ok());
        while let Some(mut stream) = connections.next().await {
            if let Err(e) = self.connect_loop(&mut stream).await {
                error!("error handling request: {}", e)
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
        return Ok(RequestState::Rejected {
            command: AuthCommand::ConnectRequest,
            reason: ReturnCode::VersionInvalid,
        });
    };

    let mut buffer = [0u8; 16];
    let username = {
        let username = &mut buffer[..request.identifier_length as usize];
        stream.read(username).await?;
        match str::from_utf8(username) {
            Ok(s) => s,
            Err(e) => {
                debug!("user connected with invalid username: {}", e);
                return Ok(RequestState::Rejected {
                    command: AuthCommand::ConnectRequest,
                    reason: ReturnCode::Failed,
                });
            }
        }
    };

    debug!("auth challenge for {}", username);

    let (state, response) = match accounts.initiate_login(username).await {
        Ok(token) => (RequestState::ConnectChallenge { token }, token.into()),
        Err(reason) => {
            return Ok(RequestState::Rejected {
                command: AuthCommand::ConnectRequest,
                reason: reason.into(),
            });
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
            return Ok(RequestState::Rejected {
            command: AuthCommand::AuthLogonProof,
            reason: status.into(),
        });
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
        return Ok(RequestState::Rejected {
            command: AuthCommand::AuthReconnectChallenge,
            reason: ReturnCode::VersionInvalid,
        });
    }

    let mut buffer = [0u8; 16];
    let username = {
        let username = &mut buffer[..request.identifier_length as usize];
        stream.read(username).await?;
        match str::from_utf8(username) {
            Ok(s) => s,
            Err(e) => {
                debug!("user connected with invalid username: {}", e);
                return Ok(RequestState::Rejected {
                    command: AuthCommand::AuthReconnectChallenge,
                    reason: ReturnCode::Failed,
                });
            }
        }
    };

    let token = match accounts.initiate_relogin(username).await {
        Ok(token) => token,
        Err(e) => {
            return Ok(RequestState::Rejected {
                command: AuthCommand::AuthReconnectChallenge,
                reason: e.into(),
            })
        }
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
            return Ok(RequestState::Rejected {
                command: AuthCommand::AuthReconnectChallenge,
                reason: status.into(),
            });
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
