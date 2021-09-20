use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use async_std::{
    channel::{unbounded, Receiver, Sender},
    net::TcpStream,
    stream::{interval, Interval},
    sync::RwLock,
};



use super::Session;
use crate::{
    client::{Client, ClientId},
    protocol::{Addon, ClientPacket, ServerPacket},
};

pub struct World {
    receiver: Receiver<(ClientId, ClientPacket)>,
    sender: Sender<(ClientId, ClientPacket)>,
    sessions: Arc<RwLock<HashMap<ClientId, Arc<Session>>>>,
}

impl World {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        World {
            sender,
            receiver,
            sessions: Default::default(),
        }
    }

    /// runs background tasks
    pub async fn timers(&self) -> Result<()> {
        let _timers = WorldTimers::new();
        // todo(arlyon): update uptime and population
        // todo(arlyon): ping database

        Ok(())
    }

    pub async fn handle_packets(&self) -> Result<()> {
        loop {
            let (id, packet) = self.receiver.recv().await?;
            let client = {
                let sessions = self.sessions.read().await;
                match sessions.get(&id).cloned() {
                    Some(c) => c,
                    None => continue,
                }
            };
            match packet {
                ClientPacket::AuthSession(_) => {} // ignore
                ClientPacket::KeepAlive => {
                    client.reset_timeout().await;
                }
                ClientPacket::Ping { latency, ping } => {
                    client.set_latency(latency).await;
                    client.send_packet(&ServerPacket::Pong(ping)).await;
                }

                ClientPacket::ReadyForAccountDataTimes => {
                    // todo
                }
                ClientPacket::CharEnum => {
                    // get client
                    // get account

                    // get characters
                }
                ClientPacket::RealmSplit { realm: _ } => {}
            }
        }
    }

    /// updates the world
    pub async fn update(&self, _diff: Duration) {
        // update game time
    }

    pub async fn create_session(
        &self,
        client: Arc<RwLock<Client>>,
        stream: TcpStream,
        session_key: [u8; 40],
        addons: Vec<Addon>,
    ) -> Result<Arc<Session>> {
        let session =
            Arc::new(Session::new(client, stream, session_key, self.sender.clone(), addons).await?);
        self.sessions
            .write()
            .await
            .insert(session.client_id, session.clone());
        Ok(session)
    }
}

struct WorldTimers {
    uptime: Interval,
    ping_db: Interval,
}

impl WorldTimers {
    fn new() -> Self {
        Self {
            uptime: interval(Duration::from_secs(60)),
            ping_db: interval(Duration::from_secs(60 * 10)),
        }
    }
}
