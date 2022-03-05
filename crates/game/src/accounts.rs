//! accounts
//!
//! The accounts module handles the basic manipulation
//! of accounts such as login and creation / deletion.

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_more::Display;
use rand::Rng;
use sha1::Digest;
use sqlx::Type;
use thiserror::Error;
use wow_srp::{Salt, Verifier, WowSRPServer};

/// An id for an account.
#[derive(Debug, Display, PartialEq, Type, Clone, Copy)]
#[sqlx(transparent)]
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

    pub joindate: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub online: u8,
}

/// Models the status of someone's ban.
#[derive(PartialEq, Eq, Debug, Type, Clone, Copy)]
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
    pub fn accept(
        &self,
        public_key: &[u8; 32],
        client_proof: &[u8; 20],
    ) -> Result<([u8; 20], [u8; 40]), LoginFailure> {
        self.server
            .verify_challenge_response(public_key, client_proof)
            .map(|session_key| {
                (
                    self.server
                        .get_server_proof(public_key, client_proof, &session_key),
                    session_key,
                )
            })
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
            reconnect_proof: rand::thread_rng().gen(),
            account,
            session_key,
        }
    }

    pub fn accept(
        &self,
        proof_data: &[u8; 16],
        client_proof: &[u8; 20],
    ) -> Result<(), LoginFailure> {
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
#[async_trait]
pub trait AccountService: Send + Sync {
    async fn list_account(&self) -> Result<Vec<Account>, AccountFetchError>;

    /// Creates a new account in the system.
    async fn create_account(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<AccountId, AccountOpError>;

    /// Deletes an account from the system along with all associated
    /// information and characters.
    async fn delete_account(&self, id: AccountId) -> Result<(), AccountOpError>;

    async fn get(&self, id: AccountId) -> Result<Account, AccountOpError>;

    /// Gets an account from the system by its username.
    async fn get_by_username(&self, username: &str) -> Result<Option<Account>, AccountOpError>;

    /// Start a login in the system. This function returns a LoginVerifier
    /// which can be used to handle the second stage of the login.
    async fn initiate_login(&self, username: &str) -> Result<ConnectToken, LoginFailure>;

    /// Logs the user in with the given public key and proof.
    async fn complete_login(
        &self,
        token: &ConnectToken,
        public_key: &[u8; 32],
        proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure>;

    async fn initiate_relogin(&self, username: &str) -> Result<ReconnectToken, LoginFailure>;

    async fn complete_relogin(
        &self,
        token: &ReconnectToken,
        proof_data: &[u8; 16],
        client_proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure>;

    async fn set_ban(
        &self,
        id: AccountId,
        author: &str,
        duration: Option<Duration>,
        reason: Option<&str>,
    ) -> Result<(), AccountOpError>;
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
