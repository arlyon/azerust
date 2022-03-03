use std::{
    iter,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use azerust_game::characters::Character;
use azerust_protocol::{
    header_crypto::HeaderCrypto,
    world::{OpCode, ResponseCode},
    Addon, ClientPacket, ClientVersion, ServerPacket,
};
use bincode::Options;
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::OwnedWriteHalf, TcpStream},
    sync::{mpsc::UnboundedSender as Sender, Mutex, RwLock},
};
use tracing::trace;

use crate::{
    client::{Client, ClientId},
    world::world::GLOBAL_CACHE_MASK,
    wow_bincode::wow_bincode,
};

/// An active session in the world.
pub struct Session {
    /// keep the client id so we don't have to open the lock
    pub client_id: ClientId,
    pub client: Arc<RwLock<Client>>,
    stream: Mutex<OwnedWriteHalf>,
    encryption: Mutex<HeaderCrypto>,
    sender: Sender<(ClientId, ClientPacket)>,
    latency: AtomicU32,
    timeout: Mutex<Instant>,
    addons: Vec<Addon>,

    character: Arc<RwLock<Option<Character>>>,
}

impl Session {
    pub async fn new(
        client: Arc<RwLock<Client>>,
        stream: OwnedWriteHalf,
        session_key: [u8; 40],
        sender: Sender<(ClientId, ClientPacket)>,
        addons: Vec<Addon>,
    ) -> Result<Self, (anyhow::Error, OwnedWriteHalf)> {
        let client_id = client.read().await.id;
        let x = Self {
            client,
            client_id,
            stream: Mutex::new(stream),
            encryption: Mutex::new(HeaderCrypto::new(session_key)),
            sender,
            addons,
            latency: AtomicU32::new(0),
            timeout: Mutex::new(Instant::now()),
            character: Arc::new(RwLock::new(None)),
        };
        match x.finalize().await {
            Ok(_) => Ok(x),
            Err(e) => Err((e, x.stream.into_inner())),
        }
    }

    pub async fn login(&self, character: Character) -> Result<()> {
        {
            self.character.write().await.replace(character);
        }

        // todo(arlyon)

        Ok(())
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
            // .await
            .context("couldnt send packet")
    }

    /// send a packet to the client
    pub async fn send_packet(&self, p: ServerPacket) -> Result<()> {
        trace!("sending packet to client {:?}: {:?}", self.client_id, p);
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
                    &wow_bincode().serialize(&version)?,
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
                self.write_packet(OpCode::SmsgPong, &wow_bincode().serialize(&ping)?)
                    .await?;
            }
            ServerPacket::CharEnum(characters) => {
                let length = iter::once(characters.len() as u8);
                trace!(
                    "sending characters {:?}",
                    characters.iter().map(|(c, _)| &c.name).collect::<Vec<_>>()
                );
                let char_data = characters.into_iter().flat_map(|(c, items)| {
                    wow_bincode()
                        .serialize(&(
                            c.id,
                            c.name,
                            c.race,
                            c.class,
                            [
                                c.gender,
                                c.skin_color,
                                c.face,
                                c.hair_style,
                                c.hair_color,
                                c.facial_style,
                            ],
                            c.level,
                            [
                                c.zone as u32,
                                c.map as u32,
                                c.position_x as u32,
                                c.position_y as u32,
                                c.position_z as u32,
                            ],
                            0u32, // guild
                            0u32, // flags
                            0u32,
                            0u8,  // todo(arlyon): first login
                            0u32, //
                            0u32, // pet data
                            0u32, //
                            items,
                        ))
                        .expect("data is correct")
                });

                self.write_packet(
                    OpCode::SmsgCharEnum,
                    &length.chain(char_data).collect::<Vec<_>>(),
                )
                .await?;
            }
            ServerPacket::AccountDataTimes(account_data) => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

                let mut buffer = [0u8; 4 + 1 + 4 + 4 * 3];
                wow_bincode().serialize_into(
                    &mut buffer[..],
                    &(
                        now,
                        0u8,
                        GLOBAL_CACHE_MASK,
                        account_data.config.global.map(|c| c.time).unwrap_or(0),
                        account_data.bindings.global.map(|c| c.time).unwrap_or(0),
                        account_data.macros.global.map(|c| c.time).unwrap_or(0),
                    ),
                )?;

                self.write_packet(OpCode::SmsgAccountDataTimes, &buffer)
                    .await?;
            }
            ServerPacket::RealmSplit { realm } => {
                self.write_packet(
                    OpCode::SmsgRealmSplit,
                    &wow_bincode().serialize(&(
                        realm, 0u32, // split state normal
                        "01/01/01",
                    ))?,
                )
                .await?;
            }
            ServerPacket::CharacterCreate(code) => {
                self.write_packet(OpCode::SmsgCharCreate, &[code as u8])
                    .await?;
            }
            ServerPacket::CharacterDelete(code) => {
                self.write_packet(OpCode::SmsgCharDelete, &[code as u8])
                    .await?;
            }
        };
        trace!("packet sent!");

        Ok(())
    }

    pub async fn set_latency(&self, latency: u32) {
        self.latency.store(latency, Ordering::Relaxed)
    }

    pub async fn finalize(&self) -> Result<()> {
        self.send_packet(ServerPacket::AuthResponse).await?;
        self.send_packet(ServerPacket::AddonInfo(self.addons.clone()))
            .await?;
        self.send_packet(ServerPacket::ClientCacheVersion(0))
            .await?;
        self.send_packet(ServerPacket::TutorialData).await
    }

    async fn write_packet(&self, opcode: OpCode, bytes: &[u8]) -> Result<usize> {
        let mut headers = [0u8; 4];
        wow_bincode().serialize_into(
            &mut headers[..],
            &((bytes.len() as u16 + 2).swap_bytes(), opcode),
        )?;

        trace!("writing headers!");
        self.encrypt_headers(&mut headers).await;
        trace!("done!");
        let mut packet = headers.to_vec();
        packet.extend_from_slice(bytes);

        trace!("writing!");
        let out = self.stream.lock().await.write(&packet).await?;

        trace!("done");
        Ok(out)
    }

    pub async fn encrypt_headers(&self, header: &mut [u8; 4]) {
        self.encryption.lock().await.encrypt(header)
    }

    pub async fn decrypt_headers(&self, header: &mut [u8; 6]) {
        self.encryption.lock().await.decrypt(header)
    }
}
