use azerust_game::accounts::LoginFailure;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Serialize;

/// All the known opcodes
#[repr(u8)]
#[derive(TryFromPrimitive, IntoPrimitive, Debug, Serialize, PartialEq, Eq, Clone, Copy)]
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
#[derive(Serialize, IntoPrimitive, Debug, PartialEq, Eq, Clone, Copy)]
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
