use std::convert::TryInto;

use async_trait::async_trait;
use game::{
    accounts::{
        Account, AccountId, AccountOpError, AccountService, BanStatus, ConnectToken, LoginFailure,
        ReconnectToken,
    },
    types::Locale,
};
use rand::Rng;
use tracing::{debug, error, instrument};
use wow_srp::{Salt, Verifier, WowSRPServer};

#[derive(Debug, Clone)]
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

#[async_trait]
impl AccountService for MySQLAccountService {
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

    async fn initiate_login(&self, username: &str) -> Result<ConnectToken, LoginFailure> {
        let account = match self.get_account(username).await {
            Ok(Account {
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
            Ok(x) => x,
            Err(_) => {
                return Err(LoginFailure::UnknownAccount);
            }
        };

        Ok(ConnectToken::new(
            &account.username,
            account.salt,
            account.verifier,
        ))
    }

    async fn initiate_relogin(&self, username: &str) -> Result<ReconnectToken, LoginFailure> {
        let request = sqlx::query!(
            "SELECT a.id, a.username, a.locked, a.lock_country, a.last_ip, a.failed_logins, (ab.unbandate > UNIX_TIMESTAMP() OR ab.unbandate = ab.bandate) as 'is_banned: bool', (ab.unbandate = ab.bandate) as 'is_permabanned: bool', aa.SecurityLevel as security_level, a.session_key_auth as session_key FROM account a LEFT JOIN account_access aa ON a.id = aa.AccountID LEFT JOIN account_banned ab ON ab.id = a.id AND ab.active = 1 WHERE a.username = ? AND a.session_key_auth IS NOT NULL", 
            username
        ).fetch_one(&self.pool).await.map_err(|_| LoginFailure::DatabaseError)?;

        let ban_status = match (request.is_banned, request.is_permabanned) {
            (_, Some(true)) => Some(BanStatus::Permanent),
            (Some(true), _) => Some(BanStatus::Temporary),
            _ => None,
        };

        let account = Account {
            id: AccountId(request.id),
            username: request.username,
            salt: Salt([0u8; 32]),
            verifier: Verifier([0u8; 32]),
            ban_status,
        };

        // get session key

        Ok(ReconnectToken::new(
            account,
            request
                .session_key
                .and_then(|k| k.as_slice().try_into().ok())
                .ok_or(LoginFailure::DatabaseError)?,
        ))
    }

    async fn complete_login(
        &self,
        token: &ConnectToken,
        public_key: &[u8; 32],
        client_proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure> {
        let (server_proof, session_key) = token.accept(public_key, client_proof)?;

        // update session information
        // todo(arlyon) set this information
        sqlx::query!(
            "UPDATE account SET session_key_auth = ?, last_ip = ?, last_login = NOW(), locale = ?, failed_logins = 0, os = ? WHERE username = ?", 
            &session_key[..], "0.0.0.0", u8::from(Locale::enUS), "Win", "ARLYON"
        )
        .execute(&self.pool)
        .await.map_err(|e| {
            error!("error updating session: {}", e);
            LoginFailure::DatabaseError
        })?;

        Ok(server_proof)
    }

    async fn complete_relogin(
        &self,
        token: &ReconnectToken,
        proof_data: &[u8; 16],
        client_proof: &[u8; 20],
    ) -> Result<[u8; 20], LoginFailure> {
        token
            .accept(proof_data, client_proof)
            .map(|_| client_proof.to_owned())
    }
}
