use async_trait::async_trait;
use game::accounts::{
    Account, AccountId, AccountOpError, AccountService, BanStatus, LoginFailure, LoginVerifier,
};
use tracing::{debug, instrument};
use wow_srp::{Salt, WowSRPServer};

#[derive(Debug)]
pub struct MySQLAccountService {
    pool: sqlx::MySqlPool,
}

impl MySQLAccountService {
    pub async fn new(connect: &str) -> Result<Self, sqlx::Error> {
        Ok(Self {
            pool: sqlx::MySqlPool::connect(connect).await?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MySQLLoginVerifier(WowSRPServer);

impl From<Account> for MySQLLoginVerifier {
    fn from(account: Account) -> Self {
        Self(WowSRPServer::new(
            &account.username,
            account.salt,
            account.verifier,
        ))
    }
}

#[async_trait]
impl LoginVerifier for MySQLLoginVerifier {
    /// Get the g parameter in use by this server.
    fn get_g(&self) -> Vec<u8> {
        self.0.get_g()
    }

    /// Get the n parameter in use by this server.
    fn get_n(&self) -> Vec<u8> {
        self.0.get_n()
    }

    /// Get the random salt in use by this server.
    fn get_salt(&self) -> &Salt {
        self.0.get_salt()
    }

    /// Get the ephemeral public key for this server.
    fn get_b_pub(&self) -> &[u8; 32] {
        self.0.get_b_pub()
    }

    fn get_security_flags(&self) -> u8 {
        0x0
    }

    async fn login(
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

#[async_trait]
impl AccountService<MySQLLoginVerifier> for MySQLAccountService {
    #[instrument(skip(self))]
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

        let (verifier, salt) = WowSRPServer::register(&username, &password);

        let done = sqlx::query!(
            "INSERT INTO account(username, salt, verifier, reg_mail, email, joindate) VALUES(?, ?, ?, ?, ?, NOW())",
            &username, &salt.0[..], &verifier.0[..], &email, &email
        )
        .execute(&self.pool)
        .await.map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        let id = AccountId(done.last_insert_id() as u32);

        sqlx::query!(
            "INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid = account.id WHERE acctid IS NULL",
        )
        .execute(&self.pool)
        .await.map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        Ok(id)
    }

    #[instrument(skip(self))]
    async fn delete_account(&self, id: AccountId) -> Result<(), AccountOpError> {
        let exists = sqlx::query!("SELECT id FROM account WHERE id = ?", id.0,)
            .fetch_optional(&self.pool)
            .await
            .map(|r| r.is_some())
            .map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        if !exists {
            return Err(AccountOpError::InvalidAccount(id));
        };

        let characters: Vec<u8> = vec![];

        for _character in characters {
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

    #[instrument(skip(self))]
    async fn get_account(&self, username: &str) -> Result<Account, AccountOpError> {
        sqlx::query_as!(
            Account,
            r#"SELECT id as "id: _", username, salt as "salt: _", verifier as "verifier: _", NULL as "ban_status: _" FROM account WHERE username = ?"#,
            username
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AccountOpError::PersistError(e.to_string()))
    }

    async fn initiate_login(&self, username: &str) -> Result<MySQLLoginVerifier, LoginFailure> {
        let account = self.get_account(username).await.ok();
        let account = match account {
            Some(Account {
                ban_status: Some(status),
                username,
                ..
            }) => {
                debug!("banned user {} attempted to log in", username);
                return match status {
                    BanStatus::Temporary => Err(LoginFailure::Suspended),
                    BanStatus::Permanent => Err(LoginFailure::Banned),
                };
            }
            Some(x) => x,
            None => {
                return Err(LoginFailure::UnknownAccount);
            }
        };
        Ok(account.into())
    }
}
