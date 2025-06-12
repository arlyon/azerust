use std::{
    convert::TryInto,
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use azerust_game::{
    accounts::{Account, AccountFetchError, AccountId, AccountOpError, AccountService, BanStatus, ConnectToken, LoginFailure, ReconnectToken},
    types::Locale,
};
use time::{OffsetDateTime, PrimitiveDateTime};
use tracing::{debug, error, info, instrument, warn};
use wow_srp::{Salt, Verifier, WowSRPServer};

use crate::cornucopia::queries::accounts;

#[derive(Debug, Clone)]
pub struct PgAccountService {
    pool: deadpool_postgres::Pool,
}

impl PgAccountService {
    pub fn new(pool: deadpool_postgres::Pool) -> Self {
        debug!("Starting accounts service");
        Self { pool }
    }
}

impl AccountService for PgAccountService {
    async fn list_account(&self) -> Result<Vec<Account>, AccountFetchError> {
        accounts::list_account()
            .bind(&self.pool.get().await.map_err(|e| AccountFetchError::IO(e.to_string()))?)
            .map(|account| Account {
                id: AccountId(account.id as u32),
                username: account.username.to_owned(),
                email: account.email.to_owned(),
                ban_status: None,
                joindate: account.joindate,
                last_login: account.last_login,
                online: account.online,
                salt: Salt(account.salt.try_into().expect("salt is 32 bytes")),
                verifier: Verifier(account.verifier.try_into().expect("verifier is 32 bytes")),
                session_key: account.session_key_auth.map(|s| s.try_into().expect("session key is 40 bytes")),
            })
            .all()
            .await
            .map_err(|e| AccountFetchError::IO(e.to_string()))
    }

    #[instrument(skip(self))]
    async fn create_account(&self, username: &str, password: &str, email: &str) -> Result<AccountId, AccountOpError> {
        if username.len() > 16 {
            return Err(AccountOpError::UsernameTooLong);
        } else if password.len() > 16 {
            return Err(AccountOpError::PasswordTooLong);
        }

        // convert to uppercase
        let username = username.to_ascii_uppercase();
        let password = password.to_ascii_uppercase();

        let (verifier, salt) = WowSRPServer::register(&username, &password);

        let id = accounts::create_account()
            .bind(
                &self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?,
                &username,
                &salt.0,
                &verifier.0,
                &email,
                &email,
            )
            .await
            .map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        let id = AccountId(id as u32);

        accounts::insert_realmchars_entry()
            .bind(&self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?)
            .await
            .map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        Ok(id)
    }

    #[instrument(skip(self))]
    async fn delete_account(&self, id: AccountId) -> Result<(), AccountOpError> {
        accounts::account_exists()
            .bind(&self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?, &(id.0 as i32))
            .one()
            .await
            .map_err(|_| AccountOpError::InvalidAccount(id))?;

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
    async fn get(&self, id: AccountId) -> Result<Account, AccountOpError> {
        accounts::select_account_by_id()
            .bind(&self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?, &(id.0 as i32))
            .map(|acc| Account {
                id: AccountId(acc.id as u32),
                email: acc.email.to_string(),
                username: acc.username.to_string(),
                salt: Salt(acc.salt.try_into().expect("right length")),
                verifier: Verifier(acc.verifier.try_into().expect("right length")),
                session_key: acc.session_key_auth.map(|sk| sk.try_into().expect("right length")),
                joindate: acc.joindate,
                last_login: acc.last_login,
                online: acc.online,
                ban_status: None,
            })
            .one()
            .await
            .map_err(|_| AccountOpError::InvalidAccount(id))
    }

    #[instrument(skip(self))]
    async fn get_by_username(&self, username: &str) -> Result<Option<Account>, AccountOpError> {
        let conn = self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        let Some(mut account) = accounts::select_account_by_username()
            .bind(&conn, &username)
            .map(|acc| Account {
                id: AccountId(acc.id as u32),
                email: acc.email.to_string(),
                username: acc.username.to_string(),
                salt: Salt(acc.salt.try_into().expect("right length")),
                verifier: Verifier(acc.verifier.try_into().expect("right length")),
                session_key: acc.session_key_auth.map(|sk| sk.try_into().expect("right length")),
                joindate: acc.joindate,
                last_login: acc.last_login,
                online: acc.online,
                ban_status: None,
            })
            .opt()
            .await
            .map_err(|e| {
                tracing::error!("failed to load account: {e}");
                AccountOpError::PersistError(e.to_string())
            })?
        else {
            info!("no account found");
            return Ok(None);
        };

        info!("getting ban status");

        if let Some(ban) = accounts::select_account_banned()
            .bind(
                &self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?,
                &(account.id.0),
            )
            .opt()
            .await
            .map_err(|e| {
                tracing::error!("Error persisting account: {:?}", e);
                AccountOpError::PersistError(e.to_string())
            })?
        {
            account.ban_status = match (ban.is_banned, ban.is_permabanned) {
                (_, true) => Some(BanStatus::Permanent),
                (true, _) => Some(BanStatus::Temporary),
                _ => None,
            };
        }

        Ok(Some(account))
    }

    #[instrument(skip(self))]
    async fn initiate_login(&self, username: &str) -> Result<ConnectToken, LoginFailure> {
        tracing::info!("login attempt for {username}");
        let account = self
            .get_by_username(username)
            .await
            .map_err(|_| LoginFailure::DatabaseError)?
            .ok_or(LoginFailure::UnknownAccount)?;

        match account.ban_status {
            Some(BanStatus::Permanent) => {
                warn!("permanently banned user {username} tried to login");
                Err(LoginFailure::Banned)
            }
            Some(BanStatus::Temporary) => {
                warn!("banned user {username} tried to login");
                Err(LoginFailure::Suspended)
            }
            None => Ok(ConnectToken::new(&account.username, account.salt, account.verifier)),
        }
    }

    async fn initiate_relogin(&self, username: &str) -> Result<ReconnectToken, LoginFailure> {
        let request = accounts::initiate_relogin()
            .bind(&self.pool.get().await.map_err(|_| LoginFailure::DatabaseError)?, &username)
            .one()
            .await
            .map_err(|_| LoginFailure::DatabaseError)?;

        let ban_status = match (request.is_banned, request.is_permabanned) {
            (_, true) => Some(BanStatus::Permanent),
            (true, _) => Some(BanStatus::Temporary),
            _ => None,
        };

        let account = Account {
            id: AccountId(request.id as u32),
            username: request.username,
            salt: Salt([0u8; 32]),
            verifier: Verifier([0u8; 32]),
            ban_status,

            // todo(arlyon): fill in
            session_key: None,
            email: "".to_string(),
            online: false,
            joindate: {
                let now = OffsetDateTime::now_utc();
                PrimitiveDateTime::new(now.date(), now.time())
            },
            last_login: None,
        };

        // get session key

        Ok(ReconnectToken::new(
            account,
            request.session_key.as_slice().try_into().ok().ok_or(LoginFailure::DatabaseError)?,
        ))
    }

    async fn complete_login(&self, username: String, token: &ConnectToken, public_key: &[u8; 32], client_proof: &[u8; 20]) -> Result<[u8; 20], LoginFailure> {
        let (server_proof, session_key) = token.accept(public_key, client_proof)?;

        // update session information
        // todo(arlyon) set this information
        accounts::update_session_information()
            .bind(
                &self.pool.get().await.map_err(|_| LoginFailure::DatabaseError)?,
                &session_key,
                &IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                &u8::from(Locale::enUS),
                &"Win",
                &&username,
            )
            .await
            .map_err(|e| {
                error!("error updating session: {e}");
                LoginFailure::DatabaseError
            })?;

        info!("logged in {username}");

        Ok(server_proof)
    }

    async fn complete_relogin(&self, token: &ReconnectToken, proof_data: &[u8; 16], client_proof: &[u8; 20]) -> Result<[u8; 20], LoginFailure> {
        token.accept(proof_data, client_proof).map(|_| client_proof.to_owned())
    }

    /// Set a ban for a given account.
    ///
    /// duration: the duration of the ban. If `None`, the ban is permanent.
    ///
    /// note: permabans are represented by a unbandate of the same value as the bandate.
    async fn set_ban(&self, id: AccountId, author: &str, duration: Option<Duration>, reason: Option<&str>) -> Result<(), AccountOpError> {
        let bandate = OffsetDateTime::now_utc();
        let now_pdt = PrimitiveDateTime::new(bandate.date(), bandate.time());
        let unbandate = match duration {
            Some(d) => now_pdt + d,
            None => now_pdt,
        };

        accounts::ban_user()
            .bind(
                &self.pool.get().await.map_err(|e| AccountOpError::PersistError(e.to_string()))?,
                &id.0,
                &unbandate,
                &author,
                &reason.unwrap_or(""),
            )
            .await
            .map_err(|e| AccountOpError::PersistError(e.to_string()))?;

        info!("banned {id} for {duration:?}");

        Ok(())
    }
}
