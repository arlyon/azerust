use assert_size_attribute::assert_eq_size;
use bincode::Options;
use derive_more::Display;
use game::accounts::{ConnectToken, LoginFailure};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{ser::SerializeStruct, Deserialize, Serialize};
use thiserror::Error;
use wow_srp::Salt;

use crate::wow_bincode::wow_bincode;

pub const VERSION_CHALLENGE: [u8; 16] = [
    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
];

/// All the known OpCodes
#[repr(u8)]
#[derive(
    TryFromPrimitive, IntoPrimitive, Debug, Display, Serialize, PartialEq, Eq, Clone, Copy,
)]
#[serde(into = "u8")]
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

/// All the known return codes from the API
#[repr(u8)]
#[derive(Serialize, IntoPrimitive, Debug, PartialEq, Eq, Clone, Copy, Display)]
#[serde(into = "u8")]
pub enum ReturnCode {
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
    ParentControl = 0x0F,
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

impl From<LoginFailure> for ReturnCode {
    fn from(f: LoginFailure) -> Self {
        match f {
            LoginFailure::Suspended => ReturnCode::Suspended,
            LoginFailure::Banned => ReturnCode::Banned,
            LoginFailure::UnknownAccount => ReturnCode::UnknownAccount,
            LoginFailure::IncorrectPassword => ReturnCode::IncorrectPassword,
            LoginFailure::DatabaseError => ReturnCode::Failed,
        }
    }
}

/// ConnectRequest is sent to the server by a client
/// looking to freshly connect to the server.
#[repr(packed)]
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
///
/// note: make sure to serialize this without
///       fixint encoding, as it encodes the usize
///       of the vecs as u64, not u8
///
/// For more, see <https://wowdev.wiki/CMD_AUTH_LOGON_CHALLENGE_Server>
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectChallenge {
    pub b_pub: [u8; 32],
    pub g: Vec<u8>,
    pub n: Vec<u8>,
    pub s: Salt,
    pub security_flags: u8,
}

/// Create a connect challenge packet for a given
/// connect token.
impl From<ConnectToken> for ConnectChallenge {
    fn from(token: ConnectToken) -> Self {
        Self {
            b_pub: *token.get_b_pub(),
            g: token.get_g(),
            n: token.get_n(),
            s: *token.get_salt(),
            security_flags: token.get_security_flags(),
        }
    }
}

impl Serialize for ConnectChallenge {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let len = if self.security_flags & 0x01 > 0 { 3 } else { 0 }
            + if self.security_flags & 0x02 > 0 { 5 } else { 0 }
            + if self.security_flags & 0x04 > 0 { 1 } else { 0 };

        let mut state = serializer.serialize_struct("packet", len)?;
        state.serialize_field("B", &self.b_pub)?;
        state.serialize_field("g", &self.g)?;
        state.serialize_field("N", &self.n)?;
        state.serialize_field("s", &self.s)?;
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
#[repr(packed)]
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
#[repr(packed)]
#[assert_eq_size([u8; 1 + 20 + 4 + 4 + 2])]
#[derive(PartialEq, Debug, Serialize, Copy, Clone)]
pub struct ConnectProofResponse {
    pub error: u8,
    pub server_proof: [u8; 20],
    pub account_flags: u32,
    pub survey_id: u32,
    pub login_flags: u16,
}

/// ReconnectProof is sent to the server by a client
/// in response to a ReconnectChallenge.
#[repr(packed)]
#[assert_eq_size([u8; 16 + 20 + 20 + 1])]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ReconnectProof {
    pub proof_data: [u8; 16],
    pub client_proof: [u8; 20],
    pub unknown: [u8; 20],
    pub key_count: u8,
}
/// RealmlistRequest is sent by an authenticated
/// account after the ReconnectProof is validated.
#[repr(packed)]
#[assert_eq_size([u8; 4])]
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RealmListRequest {
    pub data: [u8; 4],
}

/// RealmListResponse is returned from a RealmlistRequest
#[repr(packed)]
#[assert_eq_size([u8; 2 + 4 + 2])]
#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct RealmListResponse {
    pub packet_size: u16,
    pub data: [u8; 4],

    /// The length of the Realm list, which
    /// is appended on the back of the packet.
    pub realm_count: u16,
}

#[derive(Error, Debug)]
#[error("could not determine the size of realm {0}")]
pub struct SizeReadError(String);

impl RealmListResponse {
    pub fn from_realms(realms: &[Realm]) -> Result<Self, SizeReadError> {
        let len: u64 = realms
            .iter()
            .map(|r| {
                wow_bincode()
                    .serialized_size(r)
                    .map_err(|_| SizeReadError(r.name.clone()))
            })
            .sum::<Result<u64, SizeReadError>>()?;

        Ok(Self {
            packet_size: 4 + 2 + len as u16 + 2,
            data: [0, 0, 0, 0],
            realm_count: realms.len() as u16,
        })
    }
}

/// A single realm in the realmlist.
#[derive(Serialize, Debug)]
pub struct Realm {
    pub realm_type: u8,
    /// This is set if, for example, it is a GM world.
    pub locked: bool,
    pub flags: u8,
    pub name: String,
    pub socket: String,
    pub population: f32,
    pub character_count: u8,
    pub timezone: u8,
    pub realm_id: u8,
}

impl Realm {
    pub fn from_realm(r: &game::realms::Realm, character_count: u8, locked: bool) -> Self {
        Self {
            realm_type: r.realm_type.into(),
            locked,
            flags: r.flags,
            name: r.name.clone(),
            socket: format!("{}:{}", r.external_address, r.port),
            population: r.population,
            character_count,
            timezone: r.timezone,
            realm_id: u32::from(r.id) as u8,
        }
    }
}

/// Reply Packet wraps a message with its opcode.
#[derive(Serialize, Debug)]
pub struct ReplyPacket<T: Serialize> {
    command: AuthCommand,
    unknown: u8,
    status: ReturnCode,
    message: T,
}

impl ReplyPacket<ConnectChallenge> {
    pub fn new(message: ConnectChallenge) -> Self {
        Self {
            command: AuthCommand::ConnectRequest,
            unknown: 0,
            status: ReturnCode::Success,
            message,
        }
    }
}

impl ReplyPacket<()> {
    pub fn new(command: AuthCommand, status: ReturnCode) -> Self {
        Self {
            command,
            unknown: 0,
            status,
            message: (),
        }
    }
}

#[cfg(test)]
mod test {
    use bincode::Options;
    use chrono::Utc;
    use game::accounts::{Account, AccountId};
    use test_case::test_case;
    use wow_srp::{Salt, Verifier, WowSRPServer};

    use super::{
        AuthCommand, ConnectChallenge, ConnectProofResponse, Realm, RealmListResponse, ReplyPacket,
        ReturnCode,
    };
    use crate::wow_bincode::wow_bincode;

    #[test]
    pub fn realm_response_size() {
        let realm = Realm {
            realm_type: 0x01,
            locked: false,
            flags: 0,
            name: "Blackrock".into(),
            socket: "51.178.64.97:8095".into(),
            population: 0f32,
            character_count: 0,
            timezone: 8,
            realm_id: 2,
        };

        let data = [
            0x01, 0x00, 0x00, 0x42, 0x6c, 0x61, 0x63, 0x6b, 0x72, 0x6f, 0x63, 0x6b, 0x00, 0x35,
            0x31, 0x2e, 0x31, 0x37, 0x38, 0x2e, 0x36, 0x34, 0x2e, 0x39, 0x37, 0x3a, 0x38, 0x30,
            0x39, 0x35, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x2,
        ];

        assert_eq!(&wow_bincode().serialize(&realm).unwrap(), &data)
    }

    #[test_case( &[],  &[0x8, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0] ; "no realms")]
    #[test_case( &[Realm {
        realm_type: 0x01,
        locked: false,
        flags: 0,
        name: "Blackrock".into(),
        socket: "51.178.64.97:8095".into(),
        population: 0f32,
        character_count: 0,
        timezone: 8,
        realm_id: 3,
    }],  &[
        46, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x0,
    ] ; "a realm")]
    pub fn realmlist_response_size(realms: &[Realm], data: &[u8]) {
        let realmlist = RealmListResponse::from_realms(realms).unwrap();
        assert_eq!(&wow_bincode().serialize(&realmlist).unwrap(), &data)
    }

    #[test]
    pub fn connect_challenge_unknown_account_format() {
        let data = [0x0, 0x0, 0x4];

        let packet =
            ReplyPacket::<()>::new(AuthCommand::ConnectRequest, ReturnCode::UnknownAccount);
        assert_eq!(&wow_bincode().serialize(&packet).unwrap(), &data)
    }

    #[test]
    pub fn encode_auth_command() {
        let x = AuthCommand::AuthLogonProof;
        assert_eq!(wow_bincode().serialize(&x).unwrap(), [0x01]);
    }

    #[test]
    pub fn connect_challenge_format() {
        let data = [
            192, 43, 132, 161, 6, 98, 166, 88, 39, 75, 73, 80, 84, 249, 113, 192, 25, 201, 13, 134,
            177, 68, 175, 141, 209, 131, 57, 143, 83, 18, 127, 53, 1, 7, 32, 183, 155, 62, 42, 135,
            130, 60, 171, 143, 94, 191, 191, 142, 177, 1, 8, 83, 80, 6, 41, 139, 91, 173, 189, 91,
            83, 225, 137, 94, 100, 75, 137, 129, 99, 242, 106, 75, 46, 13, 205, 184, 188, 97, 36,
            45, 78, 143, 43, 77, 167, 10, 162, 184, 241, 203, 198, 21, 245, 94, 80, 58, 48, 151,
            20, 186, 163, 30, 153, 160, 11, 33, 87, 252, 55, 63, 179, 105, 205, 210, 241, 0,
        ];

        let salt = [
            129, 99, 242, 106, 75, 46, 13, 205, 184, 188, 97, 36, 45, 78, 143, 43, 77, 167, 10,
            162, 184, 241, 203, 198, 21, 245, 94, 80, 58, 48, 151, 20,
        ];

        assert_eq!(data[67..99], salt);

        let account = Account {
            id: AccountId(1),
            username: "ARLYON".into(),
            salt: Salt(salt),
            verifier: Verifier([
                0x20, 0x1f, 0x11, 0x7e, 0xf2, 0x47, 0x46, 0x91, 0x33, 0x39, 0x3e, 0xc4, 0xbc, 0x98,
                0xf, 0xdd, 0xa, 0x8a, 0xa7, 0x30, 0x82, 0xde, 0xa1, 0x9a, 0x20, 0x3b, 0x45, 0x4a,
                0x92, 0xd0, 0x5c, 0x88,
            ]),
            ban_status: None,

            email: "".to_string(),
            joindate: Utc::now(),
            last_login: None,
            online: 0,
        };

        let server = WowSRPServer::new(&account.username, account.salt, account.verifier);
        let message = ConnectChallenge {
            b_pub: [
                192, 43, 132, 161, 6, 98, 166, 88, 39, 75, 73, 80, 84, 249, 113, 192, 25, 201, 13,
                134, 177, 68, 175, 141, 209, 131, 57, 143, 83, 18, 127, 53,
            ],
            g: server.get_g(),
            n: server.get_n(),
            s: account.salt,
            security_flags: 0,
        };

        assert_eq!(&bincode::options().serialize(&message).unwrap(), &data)
    }

    #[test]
    pub fn proof_confirm_format() {
        let data: [u8; 32] = [
            1, 0, 177, 50, 224, 237, 37, 4, 196, 159, 100, 31, 30, 14, 198, 45, 137, 158, 228, 82,
            244, 140, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0,
        ];

        let response = (
            AuthCommand::AuthLogonProof,
            ConnectProofResponse {
                error: 0,
                server_proof: [
                    177, 50, 224, 237, 37, 4, 196, 159, 100, 31, 30, 14, 198, 45, 137, 158, 228,
                    82, 244, 140,
                ],
                account_flags: 0x00800000,
                survey_id: 0x0,
                login_flags: 0x0,
            },
        );

        assert_eq!(wow_bincode().serialize(&response).unwrap(), &data)
    }

    #[test]
    pub fn reply_packet_format() {
        let data = [0x0, 0x0, 0x0];

        let packet = ReplyPacket::<()>::new(AuthCommand::ConnectRequest, ReturnCode::Success);
        assert_eq!(&wow_bincode().serialize(&packet).unwrap(), &data)
    }

    #[test]
    pub fn ip_ban_format() {
        let data = [0x0, 0x0, 0x3];

        let packet = ReplyPacket::<()>::new(AuthCommand::ConnectRequest, ReturnCode::Banned);
        assert_eq!(&wow_bincode().serialize(&packet).unwrap(), &data)
    }
}
