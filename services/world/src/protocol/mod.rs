use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use anyhow::{anyhow, bail, Result};
use azerust_game::realms::RealmId;
use azerust_protocol::{world::OpCode, Addon, AuthSession, ClientPacket};
use bincode::Options;
use flate2::read::ZlibDecoder;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::trace;

use crate::{world::Session, wow_bincode::wow_bincode};

/// Reads one or more packets from a frame in the stream
pub async fn read_packets<R: AsyncRead + Unpin>(
    reader: &mut R,
    session: Option<&Arc<Session>>,
) -> Result<Vec<ClientPacket>> {
    let mut buffer = [0u8; 2048];
    let read_len = reader.read(&mut buffer).await?;

    if read_len == 0 {
        bail!("connection closed");
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

fn read_packet(code: OpCode, bytes: &[u8]) -> Result<ClientPacket> {
    match code {
        OpCode::CmsgAuthSession => {
            let str_end = bytes
                .iter()
                .enumerate()
                .skip(4 + 4)
                .find_map(|(i, &x)| if x == 0 { Some(i) } else { None })
                .ok_or_else(|| anyhow!("could not find end of string"))?;

            let addon_start = str_end + 1 + 4 + 4 + 4 + 4 + 4 + 8 + 20;
            let packet = &bytes[..addon_start];
            let addons = &bytes[addon_start..];

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

            let realm_id = RealmId(realm_id);

            trace!("read auth session packet for {username} on realm {realm_id:?}",);

            let addons = {
                use std::io::Read;
                let expected_size = wow_bincode().deserialize::<u32>(&addons[..4])? as usize;
                let mut decoder = ZlibDecoder::new(&addons[4..]);
                let mut unzipped = Vec::with_capacity(expected_size);
                let size = decoder.read_to_end(&mut unzipped)?;
                if size != expected_size {
                    bail!(
                        "addon data not correctly decompressed, expected length {expected_size} got {size}"
                    )
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
                            "read addon {name}, getting rest of data {:02X?}",
                            &unzipped[idx + 1..][..9]
                        );
                        let (has_sig, crc, crc2): (u8, _, _) =
                            wow_bincode().deserialize(&unzipped[idx + 1..][..9])?;
                        cursor += idx + 1 + 9;
                        trace!("read addon {name}, ending at {cursor}");
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
        OpCode::CmsgPing => {
            let (seq, latency) = wow_bincode().deserialize(bytes)?;
            Ok(ClientPacket::Ping { latency, seq })
        }
        OpCode::CmsgReadyForAccountDataTimes => Ok(ClientPacket::ReadyForAccountDataTimes),
        OpCode::CmsgCharEnum => Ok(ClientPacket::CharEnum),
        // todo(arlyon): read this from the packet
        OpCode::CmsgRealmSplit => Ok(ClientPacket::RealmSplit { realm: 1 }),
        OpCode::CmsgCharCreate => {
            let (
                name,
                race,
                class,
                gender,
                skin_color,
                face,
                hair_style,
                hair_color,
                facial_style,
                outfit,
            ) = wow_bincode().deserialize(bytes)?;

            let _: u8 = outfit; // outfit isn't used

            Ok(ClientPacket::CharacterCreate {
                name,
                race,
                class,
                gender,
                skin_color,
                face,
                hair_style,
                hair_color,
                facial_style,
            })
        }
        OpCode::CmsgPlayerLogin => Ok(ClientPacket::PlayerLogin(wow_bincode().deserialize(bytes)?)),
        OpCode::CmsgCharDelete => Ok(ClientPacket::CharacterDelete(
            wow_bincode().deserialize(bytes)?,
        )),

        OpCode::CmsgSetActiveVoiceChannel => todo!(),
        OpCode::CmsgNameQuery => todo!(),
        OpCode::CmsgPlayedTime => todo!(),
        OpCode::CmsgQueryTime => todo!(),
        OpCode::CmsgZoneupdate => todo!(),
        OpCode::CmsgRequestAccountData => todo!(),
        OpCode::CmsgUpdateAccountData => todo!(),
        OpCode::CmsgSetActionbarToggles => todo!(),
        OpCode::CmsgWorldStateUiTimerUpdate => todo!(),

        c => bail!("unsupported opcode: {:?}", c),
    }
}
