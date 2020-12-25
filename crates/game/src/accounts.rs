use arrayvec::ArrayString;
use async_trait::async_trait;
use derive_more::Display;
use sqlx::Type;
use thiserror::Error;
use wow_srp::{Salt, Verifier, WowSRPServer};

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

#[derive(Debug, Clone, PartialEq)]
pub struct LoginHandler(WowSRPServer);

impl From<Account> for LoginHandler {
    fn from(account: Account) -> Self {
        Self(WowSRPServer::new(
            &account.username,
            account.salt,
            account.verifier,
        ))
    }
}

impl LoginHandler {
    /// Get the g parameter in use by this server.
    pub fn get_g(&self) -> Vec<u8> {
        self.0.get_g()
    }

    /// Get the n parameter in use by this server.
    pub fn get_n(&self) -> Vec<u8> {
        self.0.get_n()
    }

    /// Get the random salt in use by this server.
    pub fn get_salt(&self) -> &Salt {
        self.0.get_salt()
    }

    /// Get the ephemeral public key for this server.
    pub fn get_b_pub(&self) -> &[u8; 32] {
        self.0.get_b_pub()
    }

    pub fn get_security_flags(&self) -> u8 {
        0x0
    }

    pub async fn login(
        &self,
        public_key: &[u8; 32],
        proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure> {
        self.0
            .verify_challenge_response(public_key, proof)
            .map(|session_key| self.0.get_server_proof(public_key, proof, &session_key))
            .ok_or(LoginFailure::IncorrectPassword)
    }
}

pub enum LoginFailure {
    Suspended,
    Banned,
    UnknownAccount,
    IncorrectPassword,
}

/// An account service handles all the business logic for accounts.
///
/// todo(arlyon): Push the login protocol into a 'login handler'.
#[async_trait]
pub trait AccountService {
    async fn create_account(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<AccountId, AccountOpError>;
    async fn delete_account(&self, id: AccountId) -> Result<(), AccountOpError>;
    async fn get_account(&self, username: &str) -> Result<Account, AccountOpError>;

    async fn get_id(&self, username: &str) -> Result<AccountId, AccountOpError> {
        self.get_account(username).await.map(|acc| acc.id)
    }

    async fn initiate_login(&self, username: &str) -> Result<LoginHandler, LoginFailure>;
}
