use std::{
    collections::HashMap,
    convert::TryInto,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use async_std::{
    net::{TcpListener, TcpStream, UdpSocket},
    prelude::*,
    stream,
    sync::RwLock,
};
use azerust_game::{
    accounts::AccountService,
    characters::CharacterService,
    realms::{RealmId, RealmList},
};
use azerust_protocol::{
    world::{OpCode, ResponseCode},
    AuthSession, ClientPacket,
};
use bincode::Options;
use rand::Rng;
use sha1::Digest;
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
    world: World<A, R, C>,

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
        Self {
            world: World::new(realm_id, accounts.clone(), realms.clone(), characters),
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

    pub async fn start(&self) -> Result<()> {
        self.auth_server_heartbeat()
            .try_join(self.accept_clients())
            .try_join(self.update())
            .try_join(self.world.handle_packets())
            .try_join(self.world.timers())
            .await
            .map(|_| ())
    }

    #[instrument(skip(self))]
    pub async fn auth_server_heartbeat(&self) -> Result<()> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;
        socket.connect(&self.auth_server_address).await?;

        let population = 0u32;

        let mut interval = stream::interval(Duration::from_secs(5));
        while interval.next().await.is_some() {
            trace!("sending population heartbeat {}", population);
            let mut buffer = [0u8; 6];
            wow_bincode().serialize_into(&mut buffer[..], &(0u8, self.id.0 as u8, population))?;
            if let Err(_e) = socket.send(&buffer).await {
                warn!("could not send heartbeat to {}", self.auth_server_address);
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn accept_clients(&self) -> Result<()> {
        let addr = ("0.0.0.0", 8085);
        let listener = TcpListener::bind(&addr).await?;
        let mut rng = rand::thread_rng();

        info!("listening on {:?}", &addr);

        let mut connections = listener.incoming().filter_map(|s| s.ok());
        while let Some(mut stream) = connections.next().await {
            let id = ClientId(rng.gen());
            self.clients
                .write()
                .await
                .insert(id, Arc::new(RwLock::new(Client { id, account: None })));

            let challenge: [u8; 32] = rng.gen();
            let packet = (
                42u16.swap_bytes(),
                OpCode::SmsgAuthChallenge,
                1u32,
                self.realm_seed,
                challenge,
            );
            stream.write(&wow_bincode().serialize(&packet)?).await?;

            if let Err(e) = self.connect_loop(&mut stream, id).await {
                error!("error handling request: {}", e);
            }

            self.clients.write().await.remove(&id);
        }

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update(&self) -> Result<()> {
        let mut interval = stream::interval(Duration::from_millis(self.update_interval.into()))
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
    #[instrument(skip(self, stream))]
    async fn connect_loop(&self, stream: &mut TcpStream, client_id: ClientId) -> Result<()> {
        let mut session = None;
        debug!("accepting packets from {:?}", client_id);

        loop {
            let packets = read_packets(stream, &session).await?;
            for packet in packets {
                trace!("received message {:?}", packet);
                match (&session, packet) {
                    (_, ClientPacket::AuthSession(auth_session)) => {
                        let client = self.clients.read().await.get(&client_id).cloned();
                        match handle_auth_session(
                            stream.clone(),
                            &self.world,
                            client.ok_or_else(|| anyhow!("no client with this id"))?,
                            auth_session,
                            self.id,
                            &self.realm_seed,
                            &self.accounts,
                        )
                        .await
                        {
                            Ok(s) => {
                                session = Some(s);
                            }
                            Err(c) => {
                                stream
                                    .write(&wow_bincode().serialize(&(
                                        6u16.swap_bytes(),
                                        OpCode::SmsgAuthResponse,
                                        c,
                                    ))?)
                                    .await?;
                                return Err(anyhow!("client failed to authenticate"));
                            }
                        }
                    }
                    (Some(w), packet) => {
                        w.reset_timeout().try_join(w.receive_packet(packet)).await?;
                    }
                    _ => return Err(anyhow!("unhandled state, disconnecting")),
                }
            }
        }
    }
}

async fn handle_auth_session<A: AccountService, R: RealmList, C: CharacterService>(
    stream: TcpStream,
    world: &World<A, R, C>,
    client: Arc<RwLock<Client>>,
    auth_session: AuthSession,
    realm_id: RealmId,
    realm_seed: &[u8],
    accounts: &dyn AccountService,
) -> std::result::Result<Arc<Session>, ResponseCode> {
    if auth_session.realm_id != realm_id {
        debug!(
            "user {} tried to log in to realm {}, but this is realm {:?}",
            auth_session.username, auth_session.server_id, realm_id
        );
        return Err(ResponseCode::RealmListRealmNotFound);
    }

    if client.read().await.account.is_some() {
        return Err(ResponseCode::AuthAlreadyOnline);
    }

    let account = accounts
        .get_by_username(&auth_session.username)
        .await
        .map_err(|_| ResponseCode::AuthUnknownAccount)?;
    trace!(
        "user {} connecting (build {})",
        auth_session.username,
        auth_session.build
    );

    let session_key = account
        .session_key
        .ok_or(ResponseCode::AuthSessionExpired)?;

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
        Err(ResponseCode::AuthReject)
    } else {
        trace!("user {} successfully authenticated", auth_session.username);

        client.write().await.account.replace(account.id);

        world
            .create_session(client, stream, session_key, auth_session.addons)
            .await
            .map_err(|e| {
                error!("could not create WorldSession: {}", e);
                ResponseCode::AuthSystemError
            })
    }
}
