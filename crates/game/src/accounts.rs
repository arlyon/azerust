//! accounts
//!
//! The accounts module handles the basic manipulation
//! of accounts such as login and creation / deletion.

use std::{future::Future, time::Duration};

use derive_more::Display;
use rand::Rng;
use sha1::Digest;
use thiserror::Error;
use time::PrimitiveDateTime;
use wow_login_messages::version_8::CMD_AUTH_LOGON_CHALLENGE_Server;
use wow_srp::{Salt, Verifier, WowSRPServer};

/// An id for an account.
#[derive(Debug, Display, PartialEq, Eq, Clone, Copy, Hash)]
pub struct AccountId(pub u32);

/// A basic account object.
#[derive(Debug, PartialEq, Clone)]
pub struct Account {
    pub id: AccountId,
    pub username: String,
    pub email: String,
    pub ban_status: Option<BanStatus>,

    pub salt: Salt,
    pub verifier: Verifier,
    pub session_key: Option<[u8; 40]>,

    pub joindate: PrimitiveDateTime,
    pub last_login: Option<PrimitiveDateTime>,
    pub online: bool,
}

/// Models the status of someone's ban.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum BanStatus {
    Temporary,
    Permanent,
}

#[derive(Copy, Debug, Clone, PartialEq)]
/// Handles the verification step of logging in.
pub struct ConnectToken {
    server: WowSRPServer,
    security_flags: u8,
}

impl ConnectToken {
    pub fn new(username: &str, salt: Salt, verifier: Verifier) -> Self {
        Self {
            server: WowSRPServer::new(username, salt, verifier),
            security_flags: 0,
        }
    }

    /// Get the g parameter in use by this server.
    pub fn get_g(&self) -> Vec<u8> {
        self.server.get_g()
    }

    /// Get the n parameter in use by this server.
    pub fn get_n(&self) -> Vec<u8> {
        self.server.get_n()
    }

    /// Get the random salt in use by this server.
    pub fn get_salt(&self) -> &Salt {
        self.server.get_salt()
    }

    /// Get the ephemeral public key for this server.
    pub fn get_b_pub(&self) -> &[u8; 32] {
        self.server.get_b_pub()
    }

    /// Get the security flags set for this login.
    pub fn get_security_flags(&self) -> u8 {
        self.security_flags
    }

    /// Handle the keys for the public key and proof.
    pub fn accept(&self, public_key: &[u8; 32], client_proof: &[u8; 20]) -> Result<([u8; 20], [u8; 40]), LoginFailure> {
        self.server
            .verify_challenge_response(public_key, client_proof)
            .map(|session_key| (self.server.get_server_proof(public_key, client_proof, &session_key), session_key))
            .ok_or(LoginFailure::IncorrectPassword)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReconnectToken {
    pub reconnect_proof: [u8; 16],
    pub account: Account,
    pub session_key: [u8; 40],
}

impl ReconnectToken {
    pub fn new(account: Account, session_key: [u8; 40]) -> Self {
        Self {
            reconnect_proof: rand::thread_rng().r#gen(),
            account,
            session_key,
        }
    }

    pub fn accept(&self, proof_data: &[u8; 16], client_proof: &[u8; 20]) -> Result<(), LoginFailure> {
        let mut sha = sha1::Sha1::new();
        sha.update(&self.account.username);
        sha.update(proof_data);
        sha.update(self.reconnect_proof);
        sha.update(self.session_key);
        let server_proof = sha.finalize();

        if server_proof.as_slice() == client_proof {
            Ok(())
        } else {
            Err(LoginFailure::IncorrectPassword)
        }
    }
}

/// An account service handles all the business logic for accounts.
pub trait AccountService: Send + Sync {
    fn list_account(&self) -> impl Future<Output = Result<Vec<Account>, AccountFetchError>> + Send;

    /// Creates a new account in the system.
    fn create_account(&self, username: &str, password: &str, email: &str) -> impl Future<Output = Result<AccountId, AccountOpError>> + Send;

    /// Deletes an account from the system along with all associated
    /// information and characters.
    fn delete_account(&self, id: AccountId) -> impl Future<Output = Result<(), AccountOpError>> + Send;

    fn get(&self, id: AccountId) -> impl Future<Output = Result<Account, AccountOpError>> + Send;

    /// Gets an account from the system by its username.
    fn get_by_username(&self, username: &str) -> impl Future<Output = Result<Option<Account>, AccountOpError>> + Send;

    /// Start a login in the system. This function returns a LoginVerifier
    /// which can be used to handle the second stage of the login.
    fn initiate_login(&self, username: &str) -> impl Future<Output = Result<ConnectToken, LoginFailure>> + Send;

    /// Logs the user in with the given public key and proof.
    fn complete_login(
        &self,
        username: String,
        token: &ConnectToken,
        public_key: &[u8; 32],
        proof: &[u8; 20],
    ) -> impl Future<Output = Result<[u8; 20], LoginFailure>> + Send;

    fn initiate_relogin(&self, username: &str) -> impl Future<Output = Result<ReconnectToken, LoginFailure>> + Send;

    fn complete_relogin(
        &self,
        token: &ReconnectToken,
        proof_data: &[u8; 16],
        client_proof: &[u8; 20],
    ) -> impl Future<Output = Result<[u8; 20], LoginFailure>> + Send;

    fn set_ban(&self, id: AccountId, author: &str, duration: Option<Duration>, reason: Option<&str>)
    -> impl Future<Output = Result<(), AccountOpError>> + Send;
}

/// Errors that may occur when running account operations.
#[derive(Error, Debug, Display)]
pub enum AccountOpError {
    UsernameTooLong,
    PasswordTooLong,
    PersistError(String),
    InvalidAccount(AccountId),
}

/// Errors that may occur when accessing accounts.
#[derive(Error, Debug, Display)]
pub enum AccountFetchError {
    IO(String),
}

/// Errors that may occur when logging in.
#[derive(Copy, Clone, Debug)]
pub enum LoginFailure {
    Suspended,
    Banned,
    UnknownAccount,
    IncorrectPassword,
    DatabaseError,
}

impl From<LoginFailure> for CMD_AUTH_LOGON_CHALLENGE_Server {
    fn from(value: LoginFailure) -> Self {
        match value {
            LoginFailure::Suspended => CMD_AUTH_LOGON_CHALLENGE_Server::FailSuspended,
            LoginFailure::Banned => CMD_AUTH_LOGON_CHALLENGE_Server::FailBanned,
            LoginFailure::UnknownAccount => CMD_AUTH_LOGON_CHALLENGE_Server::FailUnknownAccount,
            LoginFailure::IncorrectPassword => CMD_AUTH_LOGON_CHALLENGE_Server::FailIncorrectPassword,
            LoginFailure::DatabaseError => CMD_AUTH_LOGON_CHALLENGE_Server::FailDbBusy,
        }
    }
}
