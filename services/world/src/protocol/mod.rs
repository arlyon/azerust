use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use anyhow::{anyhow, Result};
use async_std::prelude::*;
use azerust_protocol::world::OpCode;
use bincode::Options;
use flate2::read::ZlibDecoder;
use tracing::trace;

use crate::{world::Session, wow_bincode::wow_bincode};

mod client;
mod header_crypto;
mod server;

pub use client::{Addon, AuthSession, ClientPacket};
pub use header_crypto::HeaderCrypto;
pub use server::ServerPacket;

/// Reads one or more packets from a frame in the stream
pub async fn read_packets<R: async_std::io::Read + std::fmt::Debug + Unpin>(
    stream: &mut R,
    session: &Option<Arc<Session>>,
) -> Result<Vec<ClientPacket>> {
    let mut buffer = [0u8; 2048];
    let read_len = stream.read(&mut buffer).await?;

    if read_len == 0 {
        return Err(anyhow!("connection closed"));
    }

    let mut buffer = &buffer[..read_len];
    let mut packets = Vec::new();

    while !buffer.is_empty() {
        let header = match session {
            Some(s) => {
                let mut x: [u8; 6] = buffer[..6].try_into().expect("correct len");
                s.decrypt_headers(&mut x).await;
                x
            }
            None => buffer[..6].try_into().expect("correct len"),
        };

        // the op_code is u32 little endian from the client, so just pad it out
        let (size, op_code, _): (u16, u16, u16) = wow_bincode().deserialize(&header[..])?;
        let size = size.swap_bytes() as usize; // size is big endian
        let code = OpCode::try_from(op_code)?;
        let packet = &buffer[6..size + 2]; // total packet length is size + opcode

        // move buffer along for next read
        buffer = &buffer[header.len() + packet.len()..];

        trace!(
            "read {:02X?} {:02X?} for code {:?}",
            &header[..],
            packet,
            code,
        );

        packets.push(read_packet(code, packet)?);
    }

    Ok(packets)
}

fn read_packet(code: OpCode, buffer: &[u8]) -> Result<ClientPacket> {
    match code {
        OpCode::CmsgAuthSession => {
            let str_end = buffer
                .iter()
                .enumerate()
                .skip(4 + 4)
                .find_map(|(i, &x)| if x == 0 { Some(i) } else { None })
                .ok_or_else(|| anyhow!("could not find end of string"))?;

            let addon_start = str_end + 1 + 4 + 4 + 4 + 4 + 4 + 8 + 20;
            let packet = &buffer[..addon_start];
            let addons = &buffer[addon_start..];

            let (
                build,
                server_id,
                username,
                login_server_type,
                local_challenge,
                region_id,
                battlegroup_id,
                realm_id,
                dos_response,
                client_proof,
            ) = wow_bincode().deserialize(packet)?;

            trace!(
                "read auth session packet for {} on realm {}",
                username,
                realm_id
            );

            let addons = {
                use std::io::Read;
                let expected_size = wow_bincode().deserialize::<u32>(&addons[..4])? as usize;
                let mut decoder = ZlibDecoder::new(&addons[4..]);
                let mut unzipped = Vec::with_capacity(expected_size);
                let size = decoder.read_to_end(&mut unzipped)?;
                if size != expected_size {
                    return Err(anyhow!(
                        "addon data not correctly decompressed, expected length {} got {}",
                        expected_size,
                        size
                    ));
                }

                trace!("decoded addons: {:02X?}", &unzipped);

                let mut cursor = 4;
                let addon_count: u32 = wow_bincode().deserialize(&unzipped[..cursor])?;
                (0..addon_count)
                    .map(|_| {
                        let unzipped = &unzipped[cursor..];
                        let idx = unzipped
                            .iter()
                            .position(|&x| x == 0)
                            .ok_or_else(|| anyhow!("couldnt find end of string"))?;
                        let name = std::str::from_utf8(&unzipped[..idx])?;
                        trace!(
                            "read addon {}, getting rest of data {:02X?}",
                            name,
                            &unzipped[idx + 1..][..9]
                        );
                        let (has_sig, crc, crc2): (u8, _, _) =
                            wow_bincode().deserialize(&unzipped[idx + 1..][..9])?;
                        cursor += idx + 1 + 9;
                        trace!("read addon {}, ending at {}", name, cursor);
                        Ok(Addon::new(name.to_string(), has_sig == 1, crc, crc2))
                    })
                    .collect::<Result<_>>()?
            };

            Ok(ClientPacket::AuthSession(AuthSession {
                build,
                server_id,
                username,
                local_challenge,
                login_server_type,
                battlegroup_id,
                realm_id,
                dos_response,
                region_id,
                client_proof,
                addons,
            }))
        }
        OpCode::CmsgReadyForAccountDataTimes => Ok(ClientPacket::ReadyForAccountDataTimes),
        OpCode::CmsgCharEnum => Ok(ClientPacket::CharEnum),
        // todo(arlyon): read this from the packet
        OpCode::CmsgRealmSplit => Ok(ClientPacket::RealmSplit { realm: 1 }),
        c => return Err(anyhow!("unsupported opcode: {:?}", c)),
    }
}
