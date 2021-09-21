use std::{
    iter,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Instant,
};

use anyhow::{Context, Result};
use async_std::{
    channel::Sender,
    io::WriteExt,
    net::TcpStream,
    sync::{Mutex, RwLock},
};
use azerust_protocol::{
    world::{OpCode, ResponseCode},
    ClientVersion,
};
use bincode::Options;
use tracing::trace;

use crate::{
    client::{Client, ClientId},
    protocol::{Addon, ClientPacket, HeaderCrypto, ServerPacket},
    wow_bincode::wow_bincode,
};

/// An active session in the world.
pub struct Session {
    /// keep the client id so we don't have to open the lock
    pub client_id: ClientId,
    pub client: Arc<RwLock<Client>>,
    stream: RwLock<TcpStream>,
    encryption: Mutex<HeaderCrypto>,
    sender: Sender<(ClientId, ClientPacket)>,
    latency: AtomicU32,
    timeout: Mutex<Instant>,
    addons: Vec<Addon>,
}

impl Session {
    pub async fn new(
        client: Arc<RwLock<Client>>,
        stream: TcpStream,
        session_key: [u8; 40],
        sender: Sender<(ClientId, ClientPacket)>,
        addons: Vec<Addon>,
    ) -> Result<Self> {
        let client_id = client.read().await.id;
        let x = Self {
            client,
            client_id,
            stream: RwLock::new(stream),
            encryption: Mutex::new(HeaderCrypto::new(session_key)),
            sender,
            addons,
            latency: AtomicU32::new(0),
            timeout: Mutex::new(Instant::now()),
        };
        x.finalize().await?;
        Ok(x)
    }

    pub async fn reset_timeout(&self) -> Result<()> {
        // todo(arlyon): different timeouts for in game vs character screen
        let mut x = self.timeout.lock().await;
        *x = Instant::now();
        Ok(())
    }

    /// receive a packet from the client
    pub async fn receive_packet(&self, p: ClientPacket) -> Result<()> {
        self.sender
            .send((self.client_id, p))
            .await
            .context("couldnt send packet")
    }

    /// send a packet to the client
    pub async fn send_packet(&self, p: &ServerPacket) -> Result<()> {
        match p {
            ServerPacket::AuthResponse => {
                self.write_packet(
                    OpCode::SmsgAuthResponse,
                    &wow_bincode().serialize(&(
                        ResponseCode::AuthOk,
                        0u32,
                        0u8,
                        0u8,
                        ClientVersion::Wotlk,
                    ))?,
                )
                .await?;
            }
            ServerPacket::AddonInfo(addons) => {
                let addon_data = addons.iter().flat_map(|a| {
                    let diff_pub: u8 = if a.crc != 0x4C1C776D { 1 } else { 0 };
                    [2u8, 1u8, diff_pub, 0u8, 0u8, 0u8, 0u8, 0u8]
                });
                self.write_packet(
                    OpCode::SmsgAddonInfo,
                    &addon_data.chain(iter::once(0)).collect::<Vec<_>>(),
                )
                .await?;
            }
            ServerPacket::ClientCacheVersion(version) => {
                self.write_packet(
                    OpCode::SmsgClientcacheVersion,
                    &wow_bincode().serialize(version)?,
                )
                .await?;
            }
            ServerPacket::TutorialData => {
                self.write_packet(
                    OpCode::SmsgTutorialFlags,
                    &wow_bincode().serialize(&[0u32; 8])?,
                )
                .await?;
            }
            ServerPacket::Pong(ping) => {
                self.write_packet(OpCode::SmsgPong, &wow_bincode().serialize(ping)?)
                    .await?;
            }
        };

        trace!("send {:?}", p);

        Ok(())
    }

    pub async fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::Relaxed)
    }

    pub async fn finalize(&self) -> Result<()> {
        self.send_packet(&ServerPacket::AuthResponse).await?;
        self.send_packet(&ServerPacket::AddonInfo(self.addons.clone()))
            .await?;
        self.send_packet(&ServerPacket::ClientCacheVersion(0))
            .await?;
        self.send_packet(&ServerPacket::TutorialData).await
    }

    async fn write_packet(&self, opcode: OpCode, bytes: &[u8]) -> Result<usize> {
        let mut headers = [0u8; 4];
        wow_bincode().serialize_into(
            &mut headers[..],
            &((bytes.len() as u16 + 2).swap_bytes(), opcode),
        )?;

        self.encrypt_headers(&mut headers).await;
        let mut packet = headers.to_vec();
        packet.extend_from_slice(bytes);

        Ok(self.stream.write().await.write(&packet).await?)
    }

    pub async fn encrypt_headers(&self, header: &mut [u8; 4]) {
        self.encryption.lock().await.encrypt(header)
    }

    pub async fn decrypt_headers(&self, header: &mut [u8; 6]) {
        self.encryption.lock().await.decrypt(header)
    }
}
