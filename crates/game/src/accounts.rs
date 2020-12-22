use async_trait::async_trait;
use derive_more::Display;
use thiserror::Error;

#[derive(Debug, Display)]
pub struct AccountId(pub u32);

#[derive(Debug)]
pub struct Account {
    pub id: AccountId,
    pub username: String,
    pub salt: Vec<u8>,
    pub verifier: Vec<u8>,
    pub ban_status: Option<BanStatus>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum BanStatus {
    Temporary,
    Permanent,
}

#[derive(Error, Debug, Display)]
pub enum AccountOpError {
    UsernameTooLong,
    PasswordTooLong,
    PersistError,
    InvalidAccount(AccountId),
}

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
}
