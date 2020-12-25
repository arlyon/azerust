use async_std::prelude::*;
use std::convert::TryFrom;

use derive_more::Display;
use num_enum::TryFromPrimitiveError;
use thiserror::Error;
use tracing::{instrument, trace};

use self::packets::{AuthCommand, ConnectProof, ConnectRequest, RealmListRequest, ReconnectProof};

pub mod packets;

/// The various messages that we can receive from the client.
#[repr(u8)]
#[derive(Debug, Display)]
pub enum Message {
    #[display(fmt = "ConnectRequest")]
    ConnectRequest(ConnectRequest) = 0x00,
    #[display(fmt = "AuthLogonProof")]
    ConnectProof(ConnectProof) = 0x01,

    #[display(fmt = "ReconnectRequest")]
    ReconnectRequest(ConnectRequest) = 0x02,
    #[display(fmt = "ReconnectProof")]
    ReconnectProof(ReconnectProof) = 0x03,

    #[display(fmt = "RealmList")]
    RealmList(RealmListRequest) = 0x10,
}

impl TryFrom<&[u8]> for Message {
    type Error = MessageParseError;

    /// Note: bincode is little-endian by default and as such cross platform.
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let (command, data) = (data[0], &data[1..]);
        AuthCommand::try_from(command)
            .map_err(|_| MessageParseError::InvalidCommand(command))
            .and_then(|c| match c {
                AuthCommand::ConnectRequest => bincode::deserialize(data)
                    .map(Message::ConnectRequest)
                    .map_err(Into::into),
                AuthCommand::AuthLogonProof => bincode::deserialize(data)
                    .map(Message::ConnectProof)
                    .map_err(Into::into),
                AuthCommand::AuthReconnectChallenge => bincode::deserialize(data)
                    .map(Message::ReconnectRequest)
                    .map_err(Into::into),
                AuthCommand::AuthReconnectProof => bincode::deserialize(data)
                    .map(Message::ReconnectProof)
                    .map_err(Into::into),
                AuthCommand::RealmList => Ok(Message::RealmList(Default::default())),
                _ => Err(MessageParseError::InvalidCommand(command)),
            })
    }
}

#[derive(Error, Debug, Display)]
pub enum MessageParseError {
    InvalidCommand(u8),
    DecodeError(#[from] Box<bincode::ErrorKind>),
}

#[instrument(skip(packet))]
pub async fn read_packet<R: async_std::io::Read + std::fmt::Debug + Unpin>(
    packet: &mut R,
) -> Result<Message, PacketHandleError> {
    let mut buffer = [0u8; 128];
    packet.read(&mut buffer[..1]).await?;

    let command = AuthCommand::try_from(buffer[0])?;
    let command_len = match command {
        AuthCommand::ConnectRequest => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthLogonProof => std::mem::size_of::<ConnectProof>(),
        AuthCommand::AuthReconnectChallenge => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::AuthReconnectProof => std::mem::size_of::<ReconnectProof>(),
        AuthCommand::RealmList => return Ok(Message::RealmList(Default::default())),
        c => return Err(PacketHandleError::UnsupportedCommand(c)),
    };

    let read_len = packet.read(&mut buffer[..command_len]).await?;
    trace!("read {:?} into {:02X?}", read_len, &buffer[..read_len]);

    if read_len != command_len {
        return Err(PacketHandleError::MessageLength(read_len, command_len));
    }

    match command {
        AuthCommand::ConnectRequest => bincode::deserialize(&buffer[..])
            .map(Message::ConnectRequest)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthLogonProof => bincode::deserialize(&buffer[..])
            .map(Message::ConnectProof)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthReconnectChallenge => bincode::deserialize(&buffer[..])
            .map(Message::ReconnectRequest)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        AuthCommand::AuthReconnectProof => bincode::deserialize(&buffer[..])
            .map(Message::ReconnectProof)
            .map_err(|e| PacketHandleError::MessageParse(MessageParseError::DecodeError(e))),
        _ => Err(PacketHandleError::UnsupportedCommand(command)),
    }
}

#[derive(Error, Debug)]
pub enum PacketHandleError {
    #[error("could not parse message: {0}")]
    MessageParse(#[from] MessageParseError),

    #[error("received {0}, expected {1}")]
    MessageLength(usize, usize),

    #[error("error while reading packet: {0}")]
    IoRead(#[from] async_std::io::Error),

    #[error("command is not currently supported: {0}")]
    UnsupportedCommand(AuthCommand),

    #[error("command is invalid: {0}")]
    InvalidCommand(#[from] TryFromPrimitiveError<AuthCommand>),
}
