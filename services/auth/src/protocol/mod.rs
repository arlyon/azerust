use std::convert::TryFrom;

use azerust_protocol::auth::AuthCommand;
use bincode::Options;
use derive_more::Display;
use num_enum::TryFromPrimitiveError;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::{instrument, trace};

use self::packets::{ConnectProof, ConnectRequest, RealmListRequest, ReconnectProof};
use crate::wow_bincode::wow_bincode;

pub mod packets;

/// The various messages that we can receive from the client.
#[repr(u8)]
#[derive(Debug, Display)]
pub enum Message {
    #[display(fmt = "ConnectRequest")]
    Connect(ConnectRequest) = 0x00,
    #[display(fmt = "AuthLogonProof")]
    Proof(ConnectProof) = 0x01,

    #[display(fmt = "ReconnectRequest")]
    ReConnect(ConnectRequest) = 0x02,
    #[display(fmt = "ReconnectProof")]
    ReProof(ReconnectProof) = 0x03,

    #[display(fmt = "RealmList")]
    RealmList(RealmListRequest) = 0x10,
}

impl TryFrom<&[u8]> for Message {
    type Error = MessageParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let (command, bytes) = (bytes[0], &bytes[1..]);
        let command = AuthCommand::try_from(command)
            .map_err(|_| MessageParseError::InvalidCommand(command))?;

        match command {
            AuthCommand::Connect => wow_bincode().deserialize(bytes).map(Message::Connect),
            AuthCommand::Proof => wow_bincode().deserialize(bytes).map(Message::Proof),
            AuthCommand::ReConnect => wow_bincode().deserialize(bytes).map(Message::ReConnect),
            AuthCommand::ReProof => wow_bincode().deserialize(bytes).map(Message::ReProof),
            AuthCommand::RealmList => Ok(Message::RealmList(Default::default())),
        }
        .map_err(Into::into)
    }
}

#[derive(Error, Debug, Display)]
pub enum MessageParseError {
    InvalidCommand(u8),
    DecodeError(#[from] Box<bincode::ErrorKind>),
}

#[instrument(skip(stream))]
pub async fn read_packet<R: AsyncRead + std::fmt::Debug + Unpin>(
    stream: &mut R,
) -> Result<Message, PacketHandleError> {
    let mut bytes = [0u8; 128];
    stream.read_exact(&mut bytes[..1]).await?;

    let command = AuthCommand::try_from(bytes[0])?;
    let command_len = match command {
        AuthCommand::Connect => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::Proof => std::mem::size_of::<ConnectProof>(),
        AuthCommand::ReConnect => std::mem::size_of::<ConnectRequest>(),
        AuthCommand::ReProof => std::mem::size_of::<ReconnectProof>(),
        AuthCommand::RealmList => std::mem::size_of::<RealmListRequest>(),
    };

    let bytes = &mut bytes[..command_len];
    let read_len = stream.read(bytes).await?;
    trace!(
        "read {:02X?} ({} bytes) for command {:?} ({} bytes)",
        &bytes[..read_len],
        read_len,
        command,
        command_len
    );

    if read_len != command_len {
        return Err(PacketHandleError::MessageLength(read_len, command_len));
    }

    match command {
        AuthCommand::Connect => wow_bincode().deserialize(bytes).map(Message::Connect),
        AuthCommand::Proof => wow_bincode().deserialize(bytes).map(Message::Proof),
        AuthCommand::ReConnect => wow_bincode().deserialize(bytes).map(Message::ReConnect),
        AuthCommand::ReProof => wow_bincode().deserialize(bytes).map(Message::ReProof),
        AuthCommand::RealmList => wow_bincode().deserialize(bytes).map(Message::RealmList),
    }
    .map_err(|e| MessageParseError::DecodeError(e).into())
}

#[derive(Error, Debug)]
pub enum PacketHandleError {
    #[error("could not parse message: {0}")]
    MessageParse(#[from] MessageParseError),

    #[error("received {0}, expected {1}")]
    MessageLength(usize, usize),

    #[error("error while reading packet: {0}")]
    IoRead(#[from] tokio::io::Error),

    #[error("command is invalid: {0}")]
    InvalidCommand(#[from] TryFromPrimitiveError<AuthCommand>),
}
