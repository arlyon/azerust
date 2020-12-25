use async_trait::async_trait;
use derive_more::Display;
use sqlx::Type;
use thiserror::Error;
use wow_srp::{Salt, Verifier};

#[derive(Debug, Display, PartialEq, Type, Clone, Copy)]
#[sqlx(transparent)]
pub struct AccountId(pub u32);

#[derive(Debug, PartialEq, Clone)]
pub struct Account {
    pub id: AccountId,
    pub username: String,
    pub salt: Salt,
    pub verifier: Verifier,
    pub ban_status: Option<BanStatus>,
}
#[derive(PartialEq, Eq, Debug, Type, Clone, Copy)]
#[repr(u8)]
pub enum BanStatus {
    Temporary,
    Permanent,
}

#[derive(Error, Debug, Display)]
pub enum AccountOpError {
    UsernameTooLong,
    PasswordTooLong,
    PersistError(String),
    InvalidAccount(AccountId),
}

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

    fn get_security_flags(&self) -> u8;

    async fn login(
        &self,
        public_key: &[u8; 32],
        proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure>;
}

pub enum LoginFailure {
    Suspended,
    Banned,
    UnknownAccount,
    IncorrectPassword,
}

/// An account service handles all the business logic for accounts.
#[async_trait]
pub trait AccountService<H: LoginVerifier> {
    async fn create_account(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<AccountId, AccountOpError>;

    async fn delete_account(&self, id: AccountId) -> Result<(), AccountOpError>;

    async fn get_account(&self, username: &str) -> Result<Account, AccountOpError>;

    async fn initiate_login(&self, username: &str) -> Result<H, LoginFailure>;
}
