use std::{
    collections::HashMap,
    fmt, iter,
    net::Ipv4Addr,
    str,
    sync::Arc,
    time::{self, Instant},
};

use anyhow::{bail, Context, Result};
use azerust_game::{
    accounts::{AccountService, ConnectToken, ReconnectToken},
    realms::{RealmFlags, RealmList},
};
use azerust_protocol::auth::{AuthCommand, ReturnCode};
use azerust_utils::flatten;
use bincode::Options;
use derivative::Derivative;
use derive_more::Display;
use futures_util::StreamExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time::interval,
    try_join,
};
use tokio_stream::{
    iter,
    wrappers::{IntervalStream, TcpListenerStream},
};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    protocol::{
        packets::{
            ConnectChallenge, ConnectProof, ConnectProofResponse, ConnectRequest, Realm,
            RealmListResponse, ReconnectProof, ReplyPacket, VERSION_CHALLENGE,
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

    #[instrument(skip(self, host))]
    pub async fn world_server_heartbeat(&self, host: Ipv4Addr, port: u16) -> Result<()> {
        let socket = tokio::net::UdpSocket::bind((host, port)).await?;

        let mut buffer = [0u8; 6];
        loop {
            if socket.recv(&mut buffer).await.is_err() {
                warn!("received larger packet than expected");
                continue;
            };
            match wow_bincode().deserialize(&buffer) {
                Ok((0u8, realm_id, realm_pop)) => {
                    self.heartbeat
                        .write()
                        .await
                        .insert(realm_id, Instant::now());
                    trace!("got heartbeat for {realm_id} with realm pop {realm_pop}")
                }
                Ok((_, _, 0u32)) | _ => debug!("received bad buffer: {:02X?}", &buffer),
            }
        }
    }

    /// updates the realmlist based on recently received heartbeats
    #[instrument(skip(self))]
    pub async fn realmlist_updater(&self) -> Result<()> {
        let instant = iter(iter::from_fn(|| Some(Instant::now())));
        let mut interval = IntervalStream::new(interval(time::Duration::from_secs(5))).zip(instant);
        while let Some((_, now)) = interval.next().await {
            let data = {
                let mut write = self.heartbeat.write().await;
                let mut data = Vec::with_capacity(write.len());
                data.extend(
                    write
                        .drain_filter(|_, v| now.saturating_duration_since(*v).as_secs() > 15)
                        .map(|(k, _)| (k, RealmFlags::Offline)),
                );
                data.extend(write.keys().map(|&k| (k, RealmFlags::Recommended)));
                data
            };
            trace!("updating realm populations: {:?}", data);
            if let Err(r) = self.realms.update_status(data).await {
                error!("error while updating realm populations: {r}");
            }
        }
        Ok(())
    }

    #[instrument(skip(self, host, port))]
    pub async fn authentication(&self, host: Ipv4Addr, port: u16) -> Result<()> {
        let addr = (host, port);
        let listener = TcpListener::bind(&addr).await?;

        info!("listening on {:?}", &addr);

        let mut connections = TcpListenerStream::new(listener);
        while let Some(Ok(mut stream)) = connections.next().await {
            if let Err(e) = self.connect_loop(&mut stream).await {
                error!("error handling request: {e}")
            }
        }

        Ok(())
    }

    #[instrument(skip(self, stream))]
    async fn connect_loop(&self, stream: &mut TcpStream) -> Result<()> {
        let mut state = RequestState::Start;

        loop {
            let message = read_packet(stream).await?;
            debug!("received message {message} in state {state}");
            state = match (state, message) {
                (_, Message::Connect(r)) => {
                    handle_connect_request(&r, &self.accounts, stream).await?
                }
                (_, Message::ReConnect(r)) => {
                    handle_reconnect_request(&r, &self.accounts, stream).await?
                }
                (RequestState::ConnectChallenge { token }, Message::Proof(proof)) => {
                    handle_connect_proof(&proof, &self.accounts, &token, stream).await?
                }
                (RequestState::ReconnectChallenge { token }, Message::ReProof(proof)) => {
                    handle_reconnect_proof(&proof, &self.accounts, &token, stream).await?
                }
                (RequestState::Realmlist, Message::RealmList(_)) => {
                    handle_realmlist(&self.realms, stream).await?
                }
                (_, Message::Proof(_) | Message::ReProof(_)) => {
                    bail!("received proof before request")
                }
                _ => bail!("received message in bad state"),
            };

            if let RequestState::Rejected { command, reason } = state {
                let mut buffer = [0u8; 3];
                wow_bincode()
                    .serialize_into(&mut buffer[..], &ReplyPacket::<()>::new(command, reason))?;
                info!("rejecting command {:?} due to {:?}", command, reason);
                stream.write_all(&buffer).await?;
                break;
            }
        }

        Ok(())
    }
}

impl<
        T: 'static + AccountService + fmt::Debug + Send + Sync,
        R: 'static + RealmList + Send + Sync,
    > AuthServer<T, R>
{
    pub async fn start(self, host: Ipv4Addr, port: u16, heartbeat_port: u16) -> Result<()> {
        let server = Arc::new(self);

        let a = flatten(tokio::task::Builder::new().name("auth::server").spawn({
            let server = server.clone();
            async move {
                server
                    .authentication(host, port)
                    .await
                    .context("server error")
            }
        }));
        let b = flatten(tokio::task::Builder::new().name("auth::heartbeat").spawn({
            let server = server.clone();
            async move {
                server
                    .world_server_heartbeat(host, heartbeat_port)
                    .await
                    .context("hearthbeat error")
            }
        }));
        let c = flatten(tokio::task::Builder::new().name("auth::realmlist").spawn({
            let server = server.clone();
            async move { server.realmlist_updater().await.context("realmlist error") }
        }));

        try_join!(a, b, c)?;

        Ok(())
    }
}

#[instrument(skip(request, accounts, stream))]
async fn handle_connect_request(
    request: &ConnectRequest,
    accounts: &dyn AccountService,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    if request.build != 12340 {
        return Ok(RequestState::Rejected {
            command: AuthCommand::Connect,
            reason: ReturnCode::VersionInvalid,
        });
    };

    let mut buffer = [0u8; 16];
    let username = {
        let username = &mut buffer[..request.identifier_length as usize];
        stream.read_exact(username).await?;
        match str::from_utf8(username) {
            Ok(s) => s,
            Err(e) => {
                debug!("user connected with invalid username: {e}");
                return Ok(RequestState::Rejected {
                    command: AuthCommand::Connect,
                    reason: ReturnCode::Failed,
                });
            }
        }
    };

    debug!("auth challenge for {username}");

    let (state, response) = match accounts.initiate_login(username).await {
        Ok(token) => (RequestState::ConnectChallenge { token }, token.into()),
        Err(reason) => {
            return Ok(RequestState::Rejected {
                command: AuthCommand::Connect,
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

    debug!("writing {:?} ({len} bytes) for {packet:?}", &buffer[..len],);
    stream.write_all(&buffer[..len]).await?;

    Ok(state)
}

#[instrument(skip(proof, accounts, token, stream))]
async fn handle_connect_proof(
    proof: &ConnectProof,
    accounts: &dyn AccountService,
    token: &ConnectToken,
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
                command: AuthCommand::Proof,
                reason: status.into(),
            });
        }
    };

    stream
        .write_all(&wow_bincode().serialize(&(AuthCommand::Proof, response))?)
        .await?;

    Ok(state)
}

#[instrument(skip(request, accounts))]
async fn handle_reconnect_request(
    request: &ConnectRequest,
    accounts: &dyn AccountService,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    if request.build != 12340 {
        return Ok(RequestState::Rejected {
            command: AuthCommand::ReConnect,
            reason: ReturnCode::VersionInvalid,
        });
    }

    let mut buffer = [0u8; 16];
    let username = {
        let username = &mut buffer[..request.identifier_length as usize];
        stream.read_exact(username).await?;
        match str::from_utf8(username) {
            Ok(s) => s,
            Err(e) => {
                debug!("user connected with invalid username: {e}");
                return Ok(RequestState::Rejected {
                    command: AuthCommand::ReConnect,
                    reason: ReturnCode::Failed,
                });
            }
        }
    };

    let token = match accounts.initiate_relogin(username).await {
        Ok(token) => token,
        Err(e) => {
            return Ok(RequestState::Rejected {
                command: AuthCommand::ReConnect,
                reason: e.into(),
            })
        }
    };

    stream
        .write_all(&bincode::options().serialize(&(
            AuthCommand::ReConnect,
            ReturnCode::Success,
            token.reconnect_proof,
            VERSION_CHALLENGE,
        ))?)
        .await?;

    Ok(RequestState::ReconnectChallenge { token })
}

#[instrument(skip(proof, accounts, token, stream))]
async fn handle_reconnect_proof(
    proof: &ReconnectProof,
    accounts: &dyn AccountService,
    token: &ReconnectToken,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    let (state, response) = match accounts
        .complete_relogin(token, &proof.proof_data, &proof.client_proof)
        .await
    {
        Ok(_) => (
            RequestState::Realmlist,
            (AuthCommand::ReProof, ReturnCode::Success, 0u16),
        ),
        Err(status) => {
            return Ok(RequestState::Rejected {
                command: AuthCommand::ReConnect,
                reason: status.into(),
            });
        }
    };

    debug!("user has reauthenticated");
    stream.write_all(&bincode::serialize(&response)?).await?;
    Ok(state)
}

#[instrument(skip(realms, stream))]
async fn handle_realmlist(realms: &dyn RealmList, stream: &mut TcpStream) -> Result<RequestState> {
    let realms = realms
        .realms()
        .await
        .iter()
        .map(|r| Realm::from_realm(r, 0, false))
        .collect::<Vec<_>>();

    let resp = RealmListResponse::from_realms(&realms)?;
    let mut packet = Vec::with_capacity((resp.packet_size + 8).into());
    packet.append(&mut wow_bincode().serialize(&(AuthCommand::RealmList, resp))?);
    for realm in realms {
        packet.append(&mut wow_bincode().serialize(&realm)?);
    }
    packet.extend_from_slice(&[0x10, 0x0]);

    stream.write_all(&packet).await?;
    Ok(RequestState::Realmlist)
}
