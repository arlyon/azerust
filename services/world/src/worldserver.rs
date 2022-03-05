use std::{
    collections::HashMap,
    convert::TryInto,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use anyhow::{anyhow, bail, Context, Result};
use azerust_game::{
    accounts::AccountService,
    characters::CharacterService,
    realms::{RealmId, RealmList},
};
use azerust_protocol::{
    world::{OpCode, ResponseCode},
    AuthSession, ClientPacket,
};
use azerust_utils::flatten;
use bincode::Options;
use rand::Rng;
use sha1::Digest;
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::{tcp::OwnedWriteHalf, TcpListener, UdpSocket},
    sync::RwLock,
    task::JoinHandle,
    time::interval,
    try_join,
};
use tokio_stream::{
    wrappers::{IntervalStream, TcpListenerStream},
    StreamExt,
};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    client::{Client, ClientId},
    protocol::read_packets,
    world::{Session, World},
    wow_bincode::wow_bincode,
};

pub struct WorldServer<A: AccountService, R: RealmList, C: CharacterService> {
    id: RealmId,
    accounts: A,
    realms: R,
    auth_server_address: String,
    realm_seed: [u8; 4],
    clients: RwLock<HashMap<ClientId, Arc<RwLock<Client>>>>,
    pub world: World<A, R, C>,

    /// target number of milliseconds between world updates
    update_interval: u16,
    update_counter: AtomicU64,

    running: bool,
}

impl<A: AccountService + Clone, R: RealmList + Clone, C: CharacterService> WorldServer<A, R, C> {
    pub fn new(
        realm_id: RealmId,
        accounts: A,
        realms: R,
        characters: C,
        auth_server_address: String,
    ) -> Self {
        WorldServer::with_world(
            realm_id,
            accounts.clone(),
            realms.clone(),
            World::new(realm_id, accounts, realms, characters),
            auth_server_address,
        )
    }
}

impl<A: AccountService, R: RealmList, C: CharacterService> WorldServer<A, R, C> {
    pub fn with_world(
        realm_id: RealmId,
        accounts: A,
        realms: R,
        world: World<A, R, C>,
        auth_server_address: String,
    ) -> Self {
        Self {
            world,
            accounts,
            realms,
            auth_server_address,
            id: realm_id,
            realm_seed: rand::thread_rng().gen(),
            clients: Default::default(),

            update_interval: 100,
            update_counter: AtomicU64::new(0),
            running: true,
        }
    }

    /// Sends periodic heartbeat packets to the auth server
    #[instrument(skip(self))]
    pub async fn auth_server_heartbeat(&self) -> Result<()> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        socket.connect(&self.auth_server_address).await?;

        let population = 0u32;

        let mut interval = interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            trace!("sending population heartbeat {population}");
            let mut buffer = [0u8; 6];
            wow_bincode().serialize_into(&mut buffer[..], &(0u8, self.id.0 as u8, population))?;
            if let Err(_e) = socket.send(&buffer).await {
                warn!("could not send heartbeat to {}", self.auth_server_address);
            }
        }
    }

    /// Allows the world server to accept new clients
    #[instrument(skip(self))]
    pub async fn accept_clients(&self) -> Result<()> {
        let addr = ("0.0.0.0", 8085);
        let listener = TcpListener::bind(&addr).await?;

        info!("listening on {:?}", &addr);

        let mut connections = TcpListenerStream::new(listener).filter_map(|s| s.ok());
        while let Some(stream) = connections.next().await {
            let (reader, mut writer) = stream.into_split();
            let (id, challenge): (ClientId, [u8; 32]) = {
                let mut rng = rand::thread_rng();
                rng.gen()
            };

            self.clients
                .write()
                .await
                .insert(id, Arc::new(RwLock::new(Client { id, account: None })));

            let packet = (
                42u16.swap_bytes(),
                OpCode::SmsgAuthChallenge,
                1u32,
                self.realm_seed,
                challenge,
            );
            writer.write_all(&wow_bincode().serialize(&packet)?).await?;

            if let Err(e) = self.connect_loop(reader, writer, id).await {
                error!("error handling request: {e}");
            }

            self.clients.write().await.remove(&id);
        }

        Ok(())
    }

    /// Runs the world update tick
    #[instrument(skip(self))]
    pub async fn update(&self) -> Result<()> {
        let mut interval =
            IntervalStream::new(interval(Duration::from_millis(self.update_interval.into())))
                .take_while(|_| self.running);

        let mut prev_time = Instant::now();
        while interval.next().await.is_some() {
            self.update_counter.fetch_add(1, Ordering::Relaxed);
            let curr_time = Instant::now();
            self.world.update(curr_time.duration_since(prev_time)).await;

            // todo(arlyon): config update time reporting

            prev_time = curr_time;
        }

        trace!("stopping world update loop");

        Ok(())
    }

    /// handles authentication, WorldSession creation,
    /// and pipes packets into the World
    #[instrument(skip(self, reader, writer))]
    async fn connect_loop<Read>(
        &self,
        mut reader: Read,
        writer: OwnedWriteHalf,
        client_id: ClientId,
    ) -> Result<()>
    where
        Read: AsyncRead + Unpin,
    {
        debug!("accepting packets from {:?}", client_id);
        let mut packets = read_packets(&mut reader, None).await?;

        let session = match packets.drain(..1).next() {
            Some(ClientPacket::AuthSession(auth_session)) => {
                let client = self.clients.read().await.get(&client_id).cloned();
                match handle_auth_session(
                    writer,
                    &self.world,
                    client.ok_or_else(|| anyhow!("no client with this id"))?,
                    auth_session,
                    self.id,
                    &self.realm_seed,
                    &self.accounts,
                )
                .await
                {
                    Ok(s) => s,
                    Err((c, mut writer)) => {
                        writer
                            .write_all(&wow_bincode().serialize(&(
                                6u16.swap_bytes(),
                                OpCode::SmsgAuthResponse,
                                c,
                            ))?)
                            .await?;
                        bail!("client failed to authenticate")
                    }
                }
            }
            _ => bail!("client sent invalid packet"),
        };

        loop {
            for packet in packets {
                trace!("received message {:?}", packet);
                try_join!(session.reset_timeout(), session.receive_packet(packet))?;
            }

            packets = read_packets(&mut reader, Some(&session)).await?;
        }
    }
}

impl<
        A: 'static + AccountService + Send + Sync + Clone,
        R: 'static + RealmList + Send + Sync + Clone,
        C: 'static + CharacterService + Send + Sync,
    > WorldServer<A, R, C>
{
    /// Start the world server, running the various tasks that it is comprised of
    pub async fn start(self) -> Result<()> {
        let server = Arc::new(self);

        try_join!(
            flatten(tokio::task::Builder::new().name("world::heartbeat").spawn({
                let cloned = server.clone();
                async move {
                    cloned
                        .auth_server_heartbeat()
                        .await
                        .context("heartbeat error")
                }
            })),
            flatten(tokio::task::Builder::new().name("world::clients").spawn({
                let cloned = server.clone();
                async move { cloned.accept_clients().await.context("client error") }
            })),
            flatten(tokio::task::Builder::new().name("world::update").spawn({
                let cloned = server.clone();
                async move { cloned.update().await.context("update error") }
            })),
            flatten(tokio::task::Builder::new().name("world::packets").spawn({
                let cloned = server.clone();
                async move { cloned.world.handle_packets().await.context("packet error") }
            })),
            flatten(tokio::task::Builder::new().name("world::timers").spawn({
                let cloned = server.clone();
                async move { cloned.world.timers().await.context("timer error") }
            }))
        )?;

        Ok(())
    }
}

/// Handles authentication, creating a WorldSession. In the event of
/// error, returns ownership of the writer to the caller.
async fn handle_auth_session<A: AccountService, R: RealmList, C: CharacterService>(
    writer: OwnedWriteHalf,
    world: &World<A, R, C>,
    client: Arc<RwLock<Client>>,
    auth_session: AuthSession,
    realm_id: RealmId,
    realm_seed: &[u8],
    accounts: &dyn AccountService,
) -> std::result::Result<Arc<Session>, (ResponseCode, OwnedWriteHalf)> {
    if auth_session.realm_id != realm_id {
        debug!(
            "user {} tried to log in to realm {}, but this is realm {realm_id:?}",
            auth_session.username, auth_session.server_id
        );
        return Err((ResponseCode::RealmListRealmNotFound, writer));
    }

    if client.read().await.account.is_some() {
        return Err((ResponseCode::AuthAlreadyOnline, writer));
    }

    let account = match accounts.get_by_username(&auth_session.username).await {
        Ok(Some(e)) => e,
        _ => return Err((ResponseCode::AuthUnknownAccount, writer)),
    };

    trace!(
        "user {} connecting (build {})",
        auth_session.username,
        auth_session.build
    );

    let session_key = match account.session_key {
        Some(k) => k,
        None => return Err((ResponseCode::AuthSessionExpired, writer)),
    };

    // todo(arlyon): add account session_key to client

    let server_proof: [u8; 20] = {
        let mut sha = sha1::Sha1::new();
        sha.update(&account.username.as_bytes());
        sha.update(&[0u8; 4]);
        sha.update(&auth_session.local_challenge);
        sha.update(realm_seed);
        sha.update(&session_key);
        sha.finalize().try_into().expect("sha1 hashes are 20 bytes")
    };

    if auth_session.client_proof != server_proof {
        return Err((ResponseCode::AuthReject, writer));
    }

    trace!("user {} successfully authenticated", auth_session.username);

    client.write().await.account.replace(account.id);

    match world
        .create_session(client, writer, session_key, auth_session.addons)
        .await
    {
        Ok(s) => Ok(s),
        Err((e, writer)) => {
            error!("could not create WorldSession: {e}");
            return Err((ResponseCode::AuthSystemError, writer));
        }
    }
}
