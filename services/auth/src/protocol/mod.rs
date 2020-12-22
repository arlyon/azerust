use std::convert::{TryFrom, TryInto};

use derive_more::Display;
use game::accounts::{Account, AccountService, BanStatus};
use num_bigint::BigUint;
use rand::prelude::*;
use srp::server::{SrpServer, UserRecord};
use thiserror::Error;
use tracing::{event, Level};

use self::packets::{
    AuthCommand, AuthLogonProof, AuthReconnectProof, ConnectChallenge, ConnectRequest, LogonProof,
};

pub mod packets;

#[repr(u8)]
#[derive(Debug)]
pub enum Message {
    AuthLogonChallenge(ConnectRequest) = 0x00,
    AuthLogonProof(AuthLogonProof) = 0x01,

    AuthReconnectChallenge(ConnectRequest) = 0x02,
    AuthReconnectProof(AuthReconnectProof) = 0x03,

    RealmList = 0x10,
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
                    .map(|d| Message::AuthLogonChallenge(d))
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
                AuthCommand::RealmList => Ok(Message::RealmList),
                _ => Err(MessageParseError::InvalidCommand(command)),
            })
    }
}

#[derive(Error, Debug, Display)]
pub enum MessageParseError {
    InvalidCommand(u8),
    DecodeError(#[from] Box<bincode::ErrorKind>),
}
