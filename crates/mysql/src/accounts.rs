use async_trait::async_trait;
use game::accounts::{Account, AccountId, AccountOpError};
use lazy_static::lazy_static;
use rand::{prelude::*, thread_rng};
use srp::client::srp_private_key;

lazy_static! {
    static ref SALT: [u8; 32] = thread_rng().gen();
}

pub struct AccountService {
    pool: sqlx::MySqlPool,
}

impl AccountService {
    pub async fn new(connect: &str) -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: sqlx::MySqlPool::connect(connect).await?,
        })
    }
}

#[async_trait]
impl game::accounts::AccountService for AccountService {
    async fn create_account(
        &self,
        username: &str,
        password: &str,
        email: &str,
    ) -> Result<AccountId, AccountOpError> {
        if username.len() > 16 {
            return Err(AccountOpError::UsernameTooLong);
        } else if password.len() > 16 {
            return Err(AccountOpError::PasswordTooLong);
        }

        // convert to uppercase
        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();

        let salt = *SALT;
        let verifier =
            srp_private_key::<sha1::Sha1>(username.as_bytes(), password.as_bytes(), &*SALT);

        let done = sqlx::query!(
            "INSERT INTO account(username, salt, verifier, reg_mail, email, joindate) VALUES(?, ?, ?, ?, ?, NOW())",
            &username, &salt[..], verifier.as_slice(), &email, &email
        )
        .execute(&self.pool)
        .await.map_err(|_| AccountOpError::PersistError)?;

        let id = AccountId(done.last_insert_id() as u32);

        sqlx::query!(
            "INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid = account.id WHERE acctid IS NULL",
        )
        .execute(&self.pool)
        .await.map_err(|_| AccountOpError::PersistError)?;

        Ok(id)
    }

    async fn delete_account(&self, id: AccountId) -> Result<(), AccountOpError> {
        let exists = sqlx::query!("SELECT id FROM account WHERE id = ?", id.0,)
            .fetch_optional(&self.pool)
            .await
            .map(|r| r.is_some())
            .map_err(|_| AccountOpError::PersistError)?;

        if !exists {
            return Err(AccountOpError::InvalidAccount(id));
        };

        let characters: Vec<u8> = vec![];

        for character in characters {
            // delete
        }

        // delete tutorials
        // delete account data
        // delete character bans

        // delete account
        // delete access history
        // delete characters
        // delete bans
        // delete muted

        Ok(())
    }

    async fn get_account(&self, username: &str) -> Result<Account, AccountOpError> {
        let verifier =
            srp_private_key::<sha1::Sha1>(username.as_bytes(), "TEST".as_bytes(), &*SALT);

        Ok(Account {
            id: AccountId(1),
            username: username.to_string(),
            salt: (*SALT).into(),
            verifier: verifier.as_slice().into(),
            ban_status: None,
        })
    }
}
