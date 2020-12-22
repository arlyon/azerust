use std::convert::TryInto;

use num_bigint::BigUint;
use rand::Rng;
use srp::server::{SrpServer, UserRecord};
use tracing::{event, instrument, Level};

use game::accounts::{Account, AccountService, BanStatus};

use crate::protocol::packets::{ConnectChallenge, ConnectRequest, LogonProof};

pub struct InitState;

pub struct LoginChallengeState;

pub struct LoginProofState;

pub struct ReconnectChallengeState;

pub struct ReconnectProofState;

pub struct ClosedState;

pub struct Machine<T> {
    state: T,
}

impl Machine<InitState> {
    pub fn new() -> Self {
        Machine { state: InitState }
    }

    /// Generate a challenge and move to the LoginChallengeState
    pub async fn submit_request(
        self,
        request: ConnectRequest,
        username: &str,
        accounts: &dyn AccountService,
    ) -> Result<Machine<LoginChallengeState>, Machine<ClosedState>> {
        event!(Level::DEBUG, "auth challenge for {}", username);

        if request.build != 12340 {
            return Err(ConnectChallenge::VersionInvalid);
        };

        // todo get user
        let user = accounts.get_account(username).await.ok();

        let user = match user {
            Some(Account {
                ban_status: Some(status),
                username,
                ..
            }) => {
                event!(Level::DEBUG, "banned user {} attempted to log in", username);
                return match status {
                    BanStatus::Temporary => Err(ConnectChallenge::Suspended),
                    BanStatus::Permanent => Err(ConnectChallenge::Banned),
                };
            }
            Some(x) => x,
            None => {
                return Err(ConnectChallenge::UnknownAccount);
            }
        };

        event!(Level::DEBUG, "got user {:?}", user);

        let group = srp::types::SrpGroup {
            n: BigUint::from_bytes_be(&[
                0x89, 0x4B, 0x64, 0x5E, 0x89, 0xE1, 0x53, 0x5B, 0xBD, 0xAD, 0x5B, 0x8B, 0x29, 0x06,
                0x50, 0x53, 0x08, 0x01, 0xB1, 0x8E, 0xBF, 0xBF, 0x5E, 0x8F, 0xAB, 0x3C, 0x82, 0x87,
                0x2A, 0x3E, 0x9B, 0xB7,
            ]),
            g: BigUint::from_bytes_be(&[7]),
        };
        let user = UserRecord {
            salt: &user.salt,
            username: user.username.as_bytes(),
            verifier: &user.verifier,
        };

        let mut b = [0u8; 64];
        let fst: [u8; 32] = rand::thread_rng().gen();
        let snd: [u8; 32] = rand::thread_rng().gen();
        b[..32].copy_from_slice(&fst);
        b[32..].copy_from_slice(&snd);

        // generate dummy a value, so we can get the B
        let a_dummy: [u8; 32] = rand::thread_rng().gen();

        let srp = SrpServer::new(&user, &a_dummy, &b, &group).expect("works");

        let proof = LogonProof {
            srp,
            group: group,
            salt: user.salt.try_into().unwrap(),
            security_flags: 0,
        };

        Ok(Machine {
            state: LoginChallengeState {},
        })
    }

    pub fn get_reconnect_challenge(self) -> Machine<ReconnectChallengeState> {}
}

impl Machine<LoginChallengeState> {
    /// Provide the proof to the challenge and move to the LoginProofState
    pub fn give_proof(
        self,
        proof: LoginProof,
    ) -> Result<Machine<LoginProofState>, Machine<ClosedState>> {
    }

    pub fn get_challenge(&self) -> ConnectChallenge {}
}

impl Machine<LoginProofState> {
    #[must_use]
    pub fn close(self) -> Machine<ClosedState> {}

    pub fn get_realmlist(&self) -> u32 {}
}

impl Machine<ReconnectChallengeState> {
    /// Provide the proof to the challenge and move to the ReconnectProofState
    pub fn give_proof(f: LoginProof) -> Result<Machine<ReconnectProofState>, Machine<ClosedState>> {
    }
}

impl Machine<ReconnectProofState> {
    pub fn get_realmlist() -> Machine<ClosedState> {}
}
