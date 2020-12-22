use std::convert::TryInto;

use assert_size_attribute::assert_eq_size;
use derivative::Derivative;
use derive_more::DebugCustom;
use num_enum::TryFromPrimitive;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use srp::{server::SrpServer, types::SrpGroup};
use tracing::{event, Level};

const VERSION_CHALLENGE: [u8; 16] = [
    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
];

/// All the known OpCodes
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

/// All the known ReturnCodes
#[repr(u8)]
#[derive(Serialize, Debug)]
pub enum ReturnCodes {
    Success = 0x00,
    Failed = 0x01,
    Failed2 = 0x02,
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

/// ConnectRequest is sent to the server by a client
/// looking to freshly connect to the server.
#[repr(packed(1), C)]
#[assert_eq_size([u8; 1 + 2 + 4 + 1 + 1 + 1 + 2 + 4 + 4 + 4 + 4 + 4 + 1])]
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
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
    pub ipv4: [u8; 4],

    /// The length of the SRP identifier,
    /// which is appended on the back of the packet.
    pub identifier_length: u8,
}

/// ConnectChallenge is sent to the client after
/// a ConnectRequest with a challenge for it to solve.
#[derive(DebugCustom, Derivative, Clone)]
#[derivative(PartialEq)]
#[debug(fmt = "{:?} {:?} {}", salt, group, security_flags)]
pub struct ConnectChallenge {
    pub salt: [u8; 32],
    #[derivative(PartialEq = "ignore")]
    pub srp: SrpServer<Sha1>,
    pub group: SrpGroup,
    pub security_flags: u8,
}

impl Serialize for ConnectChallenge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let g = self.group.g.to_bytes_le();
        let n = self.group.n.to_bytes_le();
        let s = self.salt;
        let b_pub: [u8; 32] = self.srp.get_b_pub().try_into().expect("wrong B length");

        event!(Level::TRACE, "g: {:?}", g);
        event!(Level::TRACE, "N: {:?}", n);
        event!(Level::TRACE, "s: {:?}", s);
        event!(Level::TRACE, "B: {:?}", b_pub);

        let len = if self.security_flags & 0x01 > 0 { 3 } else { 0 }
            + if self.security_flags & 0x02 > 0 { 5 } else { 0 }
            + if self.security_flags & 0x04 > 0 { 1 } else { 0 };

        let mut state = serializer.serialize_struct("packet", len)?;
        state.serialize_field("B", &b_pub)?;
        state.serialize_field("g", &g)?;
        state.serialize_field("N", &n)?;
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

/// ConnectProof is sent to the server after a
/// client receives a ConnectChallenge.
#[repr(packed(1))]
#[assert_eq_size([u8; 32 + 20 + 20 + 1 + 1])]
#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectProof {
    pub user_public_key: [u8; 32],
    pub user_proof: [u8; 20],
    pub crc_hash: [u8; 20],
    pub number_of_keys: u8,
    pub security_flags: u8,
}

/// ConnectProofResponse is sent to the client
/// after verifying a ConnectProof request.
#[derive(PartialEq)]
pub struct ConnectProofResponse {
    pub error: u8,
    pub server_proof: [u8; 32],
    pub account_flags: u32,
    pub survey_id: u32,
    pub login_flags: u16,
}

/// ReconnectProof is sent to the server by a client
/// in response to a ReconnectChallenge.
#[repr(packed(1))]
#[assert_eq_size([u8; 16 + 20 + 20 + 1])]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ReconnectProof {
    pub r_1: [u8; 16],
    pub r_2: [u8; 20],
    pub r_3: [u8; 20],
    pub key_count: u8,
}
/// RealmlistRequest is sent by an authenticated
/// account after the ReconnectProof is validated.
#[repr(packed(1))]
#[assert_eq_size([u8; 4])]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RealmListRequest {
    pub data: [u8; 4],
}

/// RealmListResponse is returned from a RealmlistRequest
#[repr(packed(1))]
#[assert_eq_size([u8; 4])]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RealmListResponse {
    pub packet_size: u16,
    pub data: [u8; 4],

    /// The length of the Realm list, which
    /// is appended on the back of the packet.
    pub realm_count: u16,
}

/// A single realm in the realmlist. In the
pub struct Realm {
    realm_type: u8,
    status: u8,
    color: u8,
    name: String,
    socket: String,
    pop_level: u32,
    character_count: u8,
    timezone: u8,
}

/// Reply Packet wraps a message with its opcode.
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
