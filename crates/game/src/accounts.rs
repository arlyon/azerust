//! accounts
//!
//! The accounts module handles the basic manipulation
//! of accounts such as login and creation / deletion.

use async_trait::async_trait;
use derive_more::Display;
use sqlx::Type;
use thiserror::Error;
use wow_srp::{Salt, Verifier};

/// An id for an account.
#[derive(Debug, Display, PartialEq, Type, Clone, Copy)]
#[sqlx(transparent)]
pub struct AccountId(pub u32);

/// A basic account object.
#[derive(Debug, PartialEq, Clone)]
pub struct Account {
    pub id: AccountId,
    pub username: String,
    pub salt: Salt,
    pub verifier: Verifier,
    pub ban_status: Option<BanStatus>,
}

/// Models the status of someone's ban.
#[derive(PartialEq, Eq, Debug, Type, Clone, Copy)]
#[repr(u8)]
pub enum BanStatus {
    Temporary,
    Permanent,
}

/// Handles the verification step of logging in.
#[async_trait]
pub trait LoginVerifier {
    /// Get the g parameter in use by this server.
    fn get_g(&self) -> Vec<u8>;

    /// Get the n parameter in use by this server.
    fn get_n(&self) -> Vec<u8>;

    /// Get the random salt in use by this server.
    fn get_salt(&self) -> &Salt;

    /// Get the ephemeral public key for this server.
    fn get_b_pub(&self) -> &[u8; 32];

    /// Get the security flags set for this login.
    fn get_security_flags(&self) -> u8;

    /// Logs the user in with the given public key and proof.
    async fn login(
        &self,
        public_key: &[u8; 32],
        proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure>;
}

/// An account service handles all the business logic for accounts.
#[async_trait]
pub trait AccountService<H: LoginVerifier> {
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

    /// Gets an account from the system by its username.
    async fn get_account(&self, username: &str) -> Result<Account, AccountOpError>;

    /// Start a login in the system. This function returns a LoginVerifier
    /// which can be used to handle the second stage of the login.
    async fn initiate_login(&self, username: &str) -> Result<H, LoginFailure>;
}

/// Errors that may occur when running account operations.
#[derive(Error, Debug, Display)]
pub enum AccountOpError {
    UsernameTooLong,
    PasswordTooLong,
    PersistError(String),
    InvalidAccount(AccountId),
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
