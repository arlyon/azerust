use std::convert::TryFrom;

use derive_more::Display;
use thiserror::Error;

use self::packets::{AuthCommand, ConnectProof, ConnectRequest, RealmListRequest, ReconnectProof};

pub mod packets;

#[repr(u8)]
#[derive(Debug)]
pub enum Message {
    ConnectRequest(ConnectRequest) = 0x00,
    AuthLogonProof(ConnectProof) = 0x01,

    AuthReconnectChallenge(ConnectRequest) = 0x02,
    AuthReconnectProof(ReconnectProof) = 0x03,

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
                    .map(|d| Message::ConnectRequest(d))
                    .map_err(Into::into),
                AuthCommand::AuthLogonProof => bincode::deserialize(data)
                    .map(|d| Message::AuthLogonProof(d))
                    .map_err(Into::into),
                AuthCommand::AuthReconnectChallenge => bincode::deserialize(data)
                    .map(|d| Message::AuthReconnectChallenge(d))
                    .map_err(Into::into),
                AuthCommand::AuthReconnectProof => bincode::deserialize(data)
                    .map(|d| Message::AuthReconnectProof(d))
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
