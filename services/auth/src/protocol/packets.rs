use std::{convert::TryInto, net::Ipv4Addr};

use assert_size_attribute::assert_eq_size;
use derive_more::DebugCustom;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use srp::{server::SrpServer, types::SrpGroup};
use tracing::{event, Level};

#[repr(u8)]
#[derive(TryFromPrimitive, Debug, Serialize)]
pub enum AuthCommand {
    ConnectRequest = 0x00,
    AuthLogonProof = 0x01,
    AuthReconnectChallenge = 0x02,
    AuthReconnectProof = 0x03,
    RealmList = 0x10,
    TransferInitiate = 0x30,
    TransferData = 0x31,
    TransferAccept = 0x32,
    TransferResume = 0x33,
    TransferCancel = 0x34,
}

#[repr(u8)]
#[derive(TryFromPrimitive)]
pub enum LoginResult {
    Ok = 0x00,
    Failed = 0x01,
    Failed2 = 0x02,
    Banned = 0x03,
    UnknownAccount = 0x04,
    UnknownAccount3 = 0x05,
    Alreadyonline = 0x06,
    Notime = 0x07,
    Dbbusy = 0x08,
    Badversion = 0x09,
    DownloadFile = 0x0A,
    Failed3 = 0x0B,
    Suspended = 0x0C,
    Failed4 = 0x0D,
    Connected = 0x0E,
    Parentalcontrol = 0x0F,
    LockedEnforced = 0x10,
}

#[repr(C, u8)]
#[derive(Serialize, Debug)]
pub enum ConnectChallenge {
    Success(LogonProof) = 0x00,
    Banned = 0x03,
    UnknownAccount = 0x04,
    IncorrectPassword = 0x05,
    AlreadyOnline = 0x06,
    NoTime = 0x07,
    DbBusy = 0x08,
    VersionInvalid = 0x09,
    VersionUpdate = 0x0A,
    InvalidServer = 0x0B,
    Suspended = 0x0C,
    NoAccess = 0x0D,
    SuccessSurvey = 0x0E,
    Parentcontrol = 0x0F,
    LockedEnforced = 0x10,
    TrialEnded = 0x11,
    UseBattlenet = 0x12,
    AntiIndulgence = 0x13,
    Expired = 0x14,
    NoGameAccount = 0x15,
    Chargeback = 0x16,
    InternetGameRoomWithoutBnet = 0x17,
    GameAccountLocked = 0x18,
    UnlockableLock = 0x19,
    ConversionRequired = 0x20,
    Disconnected = 0xFF,
}

#[repr(u8)]
pub enum AuthLogonProofResponse {
    UnknownAccount = 0x04,
    VersionInvalid = 0x09,
    Success(LogonProofResponse) = 0x0,
}

const VERSION_CHALLENGE: [u8; 16] = [
    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
];

#[derive(DebugCustom)]
#[debug(fmt = "{:?} {:?} {}", salt, group, security_flags)]
pub struct LogonProof {
    pub salt: [u8; 32],
    pub srp: SrpServer<Sha1>,
    pub group: SrpGroup,
    pub security_flags: u8,
}

impl Serialize for LogonProof {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let g = self.group.g.to_bytes_le();
        let N = self.group.n.to_bytes_le();
        let s = self.salt;
        let b_pub: [u8; 32] = self.srp.get_b_pub().try_into().expect("wrong B length");

        event!(Level::TRACE, "g: {:?}", g);
        event!(Level::TRACE, "N: {:?}", N);
        event!(Level::TRACE, "s: {:?}", s);
        event!(Level::TRACE, "B: {:?}", b_pub);

        let len = if self.security_flags & 0x01 > 0 { 3 } else { 0 }
            + if self.security_flags & 0x02 > 0 { 5 } else { 0 }
            + if self.security_flags & 0x04 > 0 { 1 } else { 0 };

        let mut state = serializer.serialize_struct("packet", len)?;
        state.serialize_field("B", &b_pub)?;
        state.serialize_field("g", &g)?;
        state.serialize_field("N", &N)?;
        state.serialize_field("s", &s)?;
        state.serialize_field("challenge", &VERSION_CHALLENGE)?;
        state.serialize_field("flags", &self.security_flags)?;

        // pin
        if self.security_flags & 0x01 > 0 {
            state.serialize_field("p1", &0u32)?;
            state.serialize_field("p2", &0u64)?;
            state.serialize_field("p3", &0u64)?;
        };

        // matrix
        if self.security_flags & 0x02 > 0 {
            state.serialize_field("m1", &0u8)?;
            state.serialize_field("m2", &0u8)?;
            state.serialize_field("m3", &0u8)?;
            state.serialize_field("m4", &0u8)?;
            state.serialize_field("m5", &0u64)?;
        };

        // token
        if self.security_flags & 0x04 > 0 {
            state.serialize_field("t1", &0u8)?;
        };

        state.end()
    }
}

#[repr(packed(1))]
#[assert_eq_size([u8; 16 + 20 + 20 + 1])]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AuthReconnectProof {
    pub R1: [u8; 16],
    pub R2: [u8; 20],
    pub R3: [u8; 20],
    pub key_count: u8,
}

/// The auth logon challenge is a struct followed by a slice of I_len bytes.
#[repr(packed(1), C)]
#[assert_eq_size([u8; 1 + 2 + 4 + 1 + 1 + 1 + 2 + 4 + 4 + 4 + 4 + 4 + 1])]
#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectRequest {
    pub error: u8,
    pub size: u16,
    pub game_name: [u8; 4],
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub build: u16,
    pub platform: [u8; 4],
    pub os: [u8; 4],
    pub country: [u8; 4],
    pub timezone_bias: u32,
    pub ip: [u8; 4],
    pub I_len: u8,
}

impl ConnectRequest {
    pub fn ip(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.ip)
    }
}

#[repr(packed(1))]
#[assert_eq_size([u8; 32 + 20 + 20 + 1 + 1])]
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthLogonProof {
    pub srp_A: [u8; 32],
    pub srp_client_M: [u8; 20],
    pub crc_hash: [u8; 20],
    pub number_of_keys: u8,
    pub security_flags: u8,
}

pub struct LogonProofResponse {
    pub error: u8,
    pub M2: [u8; 32],
    pub account_flags: u32,
    pub survey_id: u32,
    pub login_flags: u16,
}

#[derive(Serialize)]
pub struct ReplyPacket {
    command: AuthCommand,
    padding: u8,
    message: ConnectChallenge,
}

impl ReplyPacket {
    pub fn new(message: ConnectChallenge) -> Self {
        Self {
            command: AuthCommand::ConnectRequest,
            padding: 0,
            message,
        }
    }
}
