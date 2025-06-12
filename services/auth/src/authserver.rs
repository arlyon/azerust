use std::{
    collections::HashMap,
    fmt, iter,
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use azerust_game::{
    accounts::{AccountService, ConnectToken, ReconnectToken},
    realms::{RealmFlags, RealmList},
};
use azerust_utils::flatten;
use derivative::Derivative;
use derive_more::Display;
use futures_util::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time::interval,
    try_join,
};
use tokio_stream::{
    iter,
    wrappers::{IntervalStream, TcpListenerStream},
};
use tokio_util::sync::CancellationToken;
use tracing::{Level, debug, error, info, instrument, trace, warn};
use wow_login_messages::{
    Message,
    all::{CMD_AUTH_LOGON_CHALLENGE_Client, CMD_AUTH_RECONNECT_CHALLENGE_Client},
    version_8::{
        self, AccountFlag, CMD_AUTH_LOGON_CHALLENGE_Server, CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag, CMD_AUTH_LOGON_PROOF_Client,
        CMD_AUTH_LOGON_PROOF_Server, CMD_AUTH_RECONNECT_CHALLENGE_Server, CMD_AUTH_RECONNECT_PROOF_Client, CMD_AUTH_RECONNECT_PROOF_Server,
        CMD_REALM_LIST_Server, LoginResult,
        opcodes::{ClientOpcodeMessage, ServerOpcodeMessage},
    },
};

pub const VERSION_CHALLENGE: [u8; 16] = [0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1];

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
        username: String,
    },

    #[display(fmt = "ReconnectChallenge")]
    ReconnectChallenge { token: ReconnectToken },

    // the server sends the challenge and gets a proof. this results
    // in either the authenticated or rejected states.
    /// The server has accepted the request.
    Realmlist,

    /// The server has rejected the request.
    #[display(fmt = "Rejected")]
    Rejected(ServerOpcodeMessage),
}

/// Implements a WoW authentication server.
#[derive(Debug)]
pub struct AuthServer<T: AccountService + fmt::Debug, R: RealmList> {
    accounts: T,
    realms: R,
    heartbeat: Arc<RwLock<HashMap<u8, Instant>>>,
}

impl<T: Clone + AccountService + fmt::Debug, R: Clone + RealmList> Clone for AuthServer<T, R> {
    fn clone(&self) -> Self {
        Self {
            accounts: self.accounts.clone(),
            realms: self.realms.clone(),
            heartbeat: self.heartbeat.clone(),
        }
    }
}

impl<T: AccountService + fmt::Debug + Clone + 'static, R: RealmList + Clone + 'static> AuthServer<T, R> {
    pub fn new(accounts: T, realms: R) -> Self {
        Self {
            accounts,
            realms,
            heartbeat: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[instrument(skip(self, host))]
    pub async fn world_server_heartbeat(&self, host: Ipv4Addr, port: u16) -> Result<()> {
        let socket = tokio::net::UdpSocket::bind((host, port)).await?;

        let mut buffer = [0u8; 6];

        fn parse([a, b, c, d, e, f]: [u8; 6]) -> (u8, u8, u32) {
            (a, b, u32::from_be_bytes([c, d, e, f]))
        }

        loop {
            if socket.recv(&mut buffer).await.is_err() {
                warn!("received larger packet than expected");
                continue;
            };

            match parse(buffer) {
                (0u8, realm_id, realm_pop) => {
                    self.heartbeat.write().await.insert(realm_id, Instant::now());
                    trace!("got heartbeat for {realm_id} with realm pop {realm_pop}")
                }
                _ => debug!("received bad buffer: {:02X?}", &buffer),
            };
        }
    }

    /// updates the realmlist based on recently received heartbeats
    #[instrument(skip(self))]
    pub async fn realmlist_updater(&self) -> Result<()> {
        let instant = iter(iter::from_fn(|| Some(Instant::now())));
        let mut interval = IntervalStream::new(interval(Duration::from_secs(5))).zip(instant);
        while let Some((_, now)) = interval.next().await {
            let span = tracing::span!(Level::TRACE, "update_realmlist");
            let _guard = span.enter();
            let data = {
                let mut write = self.heartbeat.write().await;
                let mut data = Vec::with_capacity(write.len());
                data.extend(
                    write
                        .extract_if(|_, v| now.saturating_duration_since(*v).as_secs() > 15)
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
        // let id = Span::current().id().unwrap();
        while let Some(Ok(mut stream)) = connections.next().await {
            tokio::task::Builder::new()
                .name(&format!(
                    "auth::connect[{}]",
                    stream.peer_addr().map(|a| a.to_string()).unwrap_or_else(|_| "unknown".to_string())
                ))
                .spawn({
                    let server = self.to_owned();
                    // let id = id.clone();
                    async move {
                        if let Err(e) = server.connect_loop(&mut stream).await {
                            error!("error handling request: {e}")
                        }
                    }
                })?;
        }

        Ok(())
    }

    #[instrument(skip(self, stream))]
    async fn connect_loop(&self, mut stream: &mut TcpStream) -> Result<()> {
        let mut state = RequestState::Start;

        loop {
            let message = ClientOpcodeMessage::tokio_read(&mut stream).await?;

            debug!("received message {message} in state {state}");
            state = match (state, message) {
                (_, ClientOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(r)) => handle_connect_request(&r, &self.accounts, stream).await?,
                (_, ClientOpcodeMessage::CMD_AUTH_RECONNECT_CHALLENGE(r)) => handle_reconnect_request(&r, &self.accounts, stream).await?,
                (RequestState::ConnectChallenge { token, username }, ClientOpcodeMessage::CMD_AUTH_LOGON_PROOF(proof)) => {
                    handle_connect_proof(username, &proof, &self.accounts, &token, stream).await?
                }
                (RequestState::ReconnectChallenge { token }, ClientOpcodeMessage::CMD_AUTH_RECONNECT_PROOF(proof)) => {
                    handle_reconnect_proof(&proof, &self.accounts, &token, stream).await?
                }
                (RequestState::Realmlist, ClientOpcodeMessage::CMD_REALM_LIST(_)) => handle_realmlist(&self.realms, stream).await?,
                (_, ClientOpcodeMessage::CMD_AUTH_LOGON_PROOF(_) | ClientOpcodeMessage::CMD_AUTH_RECONNECT_PROOF(_)) => {
                    bail!("received proof before request")
                }
                _ => bail!("received message in bad state"),
            };

            if let RequestState::Rejected(message) = state {
                match message {
                    ServerOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(msg) => {
                        msg.tokio_write(stream).await?;
                    }
                    ServerOpcodeMessage::CMD_AUTH_LOGON_PROOF(msg) => {
                        msg.tokio_write(stream).await?;
                    }
                    ServerOpcodeMessage::CMD_AUTH_RECONNECT_CHALLENGE(msg) => {
                        msg.tokio_write(stream).await?;
                    }
                    ServerOpcodeMessage::CMD_AUTH_RECONNECT_PROOF(msg) => {
                        msg.tokio_write(stream).await?;
                    }
                    _ => bail!("received message in bad state"),
                }

                // wait a little before closing the connection, otherwise the client will
                // display a connection error rather than our rejection
                tokio::time::sleep(Duration::from_secs(10)).await;

                break;
            }
        }

        Ok(())
    }
}

impl<T: 'static + AccountService + fmt::Debug + Send + Sync + Clone, R: 'static + RealmList + Send + Sync + Clone> AuthServer<T, R> {
    pub async fn start(self, host: Ipv4Addr, port: u16, heartbeat_port: u16, stop: CancellationToken) -> Result<()> {
        let server = Arc::new(self);

        let a = flatten(tokio::task::Builder::new().name("auth::server").spawn({
            let server = server.clone();
            async move { server.authentication(host, port).await.context("server error") }
        })?);
        let b = flatten(tokio::task::Builder::new().name("auth::heartbeat").spawn({
            let server = server.clone();
            async move { server.world_server_heartbeat(host, heartbeat_port).await.context("hearthbeat error") }
        })?);
        let c = flatten(tokio::task::Builder::new().name("auth::realmlist").spawn({
            let server = server.clone();
            async move { server.realmlist_updater().await.context("realmlist error") }
        })?);

        let d = async {
            stop.cancelled().await;
            Result::<()>::Err(anyhow::anyhow!("cancelled"))
        };

        try_join!(a, b, c, d)?;

        Ok(())
    }
}

#[instrument(skip(request, accounts, stream))]
async fn handle_connect_request(request: &CMD_AUTH_LOGON_CHALLENGE_Client, accounts: &impl AccountService, stream: &mut TcpStream) -> Result<RequestState> {
    if request.version.build != 12340 {
        return Ok(RequestState::Rejected(ServerOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(
            CMD_AUTH_LOGON_CHALLENGE_Server::FailVersionInvalid,
        )));
    };

    debug!("auth challenge for {}", request.account_name);

    let (state, response) = match accounts.initiate_login(&request.account_name).await {
        Ok(token) => (
            RequestState::ConnectChallenge {
                token,
                username: request.account_name.to_owned(),
            },
            CMD_AUTH_LOGON_CHALLENGE_Server::Success {
                security_flag: CMD_AUTH_LOGON_CHALLENGE_Server_SecurityFlag::new(token.get_security_flags(), None, None, None),
                salt: token.get_salt().0,
                server_public_key: *token.get_b_pub(),
                generator: token.get_g(),
                large_safe_prime: token.get_n(),
                crc_salt: VERSION_CHALLENGE,
            },
        ),
        Err(reason) => {
            trace!("rejecting due to {:?}", reason);
            return Ok(RequestState::Rejected(ServerOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(reason.into())));
        }
    };

    trace!("writing response");
    response.tokio_write(stream).await?;
    trace!("written");

    Ok(state)
}

#[instrument(skip(proof, accounts, token, stream))]
async fn handle_connect_proof(
    username: String,
    proof: &CMD_AUTH_LOGON_PROOF_Client,
    accounts: &impl AccountService,
    token: &ConnectToken,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    let (state, response) = match accounts.complete_login(username, token, &proof.client_public_key, &proof.client_proof).await {
        Ok(server_proof) => (RequestState::Realmlist, CMD_AUTH_LOGON_PROOF_Server::Success {
            account_flag: AccountFlag::new_propass(), // same a MANGOS
            hardware_survey_id: 0,
            unknown: 0,
            server_proof,
        }),
        Err(reason) => {
            return Ok(RequestState::Rejected(ServerOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(reason.into())));
        }
    };

    response.tokio_write(stream).await?;

    Ok(state)
}

#[instrument(skip(request, accounts))]
async fn handle_reconnect_request(
    request: &CMD_AUTH_RECONNECT_CHALLENGE_Client,
    accounts: &impl AccountService,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    if request.version.build != 12340 {
        return Ok(RequestState::Rejected(ServerOpcodeMessage::CMD_AUTH_RECONNECT_CHALLENGE(
            CMD_AUTH_RECONNECT_CHALLENGE_Server::FailVersionInvalid,
        )));
    };

    let token = match accounts.initiate_relogin(&request.account_name).await {
        Ok(token) => token,
        Err(reason) => {
            return Ok(RequestState::Rejected(ServerOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(reason.into())));
        }
    };

    CMD_AUTH_RECONNECT_CHALLENGE_Server::Success {
        challenge_data: token.reconnect_proof,
        checksum_salt: VERSION_CHALLENGE,
    }
    .tokio_write(stream)
    .await?;

    Ok(RequestState::ReconnectChallenge { token })
}

#[instrument(skip(proof, accounts, token, stream))]
async fn handle_reconnect_proof(
    proof: &CMD_AUTH_RECONNECT_PROOF_Client,
    accounts: &impl AccountService,
    token: &ReconnectToken,
    stream: &mut TcpStream,
) -> Result<RequestState> {
    let (state, response) = match accounts.complete_relogin(token, &proof.proof_data, &proof.client_proof).await {
        Ok(_) => (RequestState::Realmlist, CMD_AUTH_RECONNECT_PROOF_Server { result: LoginResult::Success }),
        Err(reason) => {
            return Ok(RequestState::Rejected(ServerOpcodeMessage::CMD_AUTH_LOGON_CHALLENGE(reason.into())));
        }
    };

    debug!("user has reauthenticated");
    response.tokio_write(stream).await?;

    Ok(state)
}

#[instrument(skip(realms, stream))]
async fn handle_realmlist(realms: &impl RealmList, stream: &mut TcpStream) -> Result<RequestState> {
    let realms = realms
        .realms()
        .await?
        .iter()
        .map(|r| version_8::Realm {
            realm_type: match r.realm_type {
                azerust_game::realms::RealmType::Normal => version_8::RealmType::PlayerVsEnvironment,
                azerust_game::realms::RealmType::PVP => version_8::RealmType::PlayerVsPlayer,
                azerust_game::realms::RealmType::RP => version_8::RealmType::Roleplaying,
                azerust_game::realms::RealmType::RPPvP => version_8::RealmType::RoleplayingPlayerVsPlayer,
            },
            address: format!("{}:{}", r.external_address, r.port),
            name: r.name.clone(),
            locked: false, // todo: calculate this
            realm_id: r.id.0 as u8,
            number_of_characters_on_realm: 0, // todo: calculate this
            category: version_8::RealmCategory::One,
            ..Default::default()
        })
        .collect::<Vec<_>>();

    tracing::trace!("sending {} realms", realms.len());

    CMD_REALM_LIST_Server { realms }.tokio_write(stream).await?;

    Ok(RequestState::Realmlist)
}
