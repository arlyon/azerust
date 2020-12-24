//! wow-srp
//!
//! This crate implements the SRP variation that is used in
//! the World of Warcraft authentication protocol. It provides
//! currently only provides a [`WowSRPServer`].

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use std::convert::TryInto;

use lazy_static::lazy_static;
use num_bigint::BigUint;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use serde::Serialize;
use sha1::{Digest, Sha1};
use sqlx::Type;

lazy_static! {
    static ref G: BigUint = BigUint::from_bytes_be(&[7]);
    static ref N: BigUint = BigUint::from_bytes_be(&[
        0x89, 0x4B, 0x64, 0x5E, 0x89, 0xE1, 0x53, 0x5B, 0xBD, 0xAD, 0x5B, 0x8B, 0x29, 0x06, 0x50,
        0x53, 0x08, 0x01, 0xB1, 0x8E, 0xBF, 0xBF, 0x5E, 0x8F, 0xAB, 0x3C, 0x82, 0x87, 0x2A, 0x3E,
        0x9B, 0xB7,
    ]);
}

/// A salt is used to prevent dictionary attacks,
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Type)]
#[sqlx(transparent)]
pub struct Salt(pub [u8; 32]);

impl Distribution<Salt> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Salt {
        let inner = rng.gen();
        Salt(inner)
    }
}

/// A verifier allows the server to check the
/// validity of a password proof provided to it
/// by a client.
#[derive(Copy, Clone, Debug, PartialEq, Type)]
#[sqlx(transparent)]
pub struct Verifier(pub [u8; 32]);

impl From<&Verifier> for BigUint {
    fn from(v: &Verifier) -> Self {
        Self::from_bytes_le(&v.0)
    }
}

impl Verifier {
    /// Create a verifier from a set of credentials and salt.
    pub fn from_credentials(username: &str, password: &str, salt: &Salt) -> Self {
        let inner = {
            let mut d = Sha1::new();
            d.update(username.as_bytes());
            d.update(":");
            d.update(password.as_bytes());
            d.finalize()
        };

        let mut hash = Sha1::new();
        hash.update(salt.0);
        hash.update(inner);
        let hash = BigUint::from_bytes_le(&hash.finalize());

        Self(
            G.modpow(&hash, &*N)
                .to_bytes_le()
                .try_into()
                .expect("correct size"),
        )
    }

    /// Create a verifier from raw bytes.
    pub fn from_raw(data: [u8; 32]) -> Self {
        Self(data)
    }
}

/// Provides the server-side functionality of the WoW
/// SRP protocol.
///
/// ```rust
/// // register a new user
/// let (verifier, salt) = WowSRPServer::register("ARLYON", "TEST");
///
/// // some time later, load the username,
/// // salt and verifier from the database.f
/// let server = WowSRPServer::new("ARLYON", salt, verifier);
/// ```
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WowSRPServer {
    salt: Salt,
    verifier: Verifier,
    identity_hash: [u8; 20],
    b: [u8; 32],
    b_pub: [u8; 32],
}

impl WowSRPServer {
    /// Create a new instance of a server verifier.
    pub fn new(username: &str, salt: Salt, verifier: Verifier) -> Self {
        let b = [
            0xF0, 0xA4, 0xBB, 0x60, 0x1C, 0xB3, 0xE5, 0x03, 0x41, 0x26, 0xD0, 0xC7, 0x95, 0x73,
            0x19, 0xD3, 0xCB, 0x0D, 0x7B, 0xD6, 0xFE, 0x2E, 0x3C, 0x9F, 0x6F, 0x0C, 0x27, 0x28,
            0x17, 0x55, 0x76, 0x1F,
        ];
        Self {
            salt,
            b_pub: Self::calculate_b_pub(&b, &verifier),
            identity_hash: Sha1::digest(username.as_bytes())
                .try_into()
                .expect("sha1 hashes are 20 bytes"),
            verifier,
            b,
        }
    }

    /// Get the g parameter in use by this server.
    pub fn get_g(&self) -> Vec<u8> {
        G.to_bytes_le()
    }

    /// Get the n parameter in use by this server.
    pub fn get_n(&self) -> Vec<u8> {
        N.to_bytes_le()
    }

    /// Get the random salt in use by this server.
    pub fn get_salt(&self) -> &Salt {
        &self.salt
    }

    /// Get the ephemeral public key for this server.
    pub fn get_b_pub(&self) -> &[u8; 32] {
        &self.b_pub
    }

    /// Does the registration calculations, returning a verifier and
    /// a salt that can be stored in the database.
    pub fn register(username: &str, password: &str) -> (Verifier, Salt) {
        let mut rng = rand::thread_rng();
        let salt = rng.gen();
        (Verifier::from_credentials(username, password, &salt), salt)
    }

    /// Get the server proof (M2). This is derived from the client
    /// public key and client proof (M1), as well as the session key.
    pub fn get_server_proof(
        &self,
        a_pub: &[u8; 32],
        client_proof: &[u8; 20],
        session_key: &[u8; 40],
    ) -> [u8; 20] {
        let mut sha = Sha1::new();
        sha.update(a_pub);
        sha.update(client_proof);
        sha.update(session_key);
        sha.finalize().try_into().expect("sha1 hashes are 20 bytes")
    }

    /// Verify the challenge response, returning a verified key if
    /// it is valid.
    pub fn verify_challenge_response(
        self,
        a_pub: &[u8; 32],
        client_m: &[u8; 20],
    ) -> Option<[u8; 40]> {
        let a_pub_num = BigUint::from_bytes_le(a_pub);
        let verifier = BigUint::from(&self.verifier);
        let b = BigUint::from_bytes_be(&self.b);

        if (&a_pub_num % &*N).eq(&0u8.into()) {
            return None;
        };

        let a_b = {
            let mut sha = Sha1::new();
            sha.update(a_pub);
            sha.update(self.b_pub);
            sha.finalize()
        };

        let u = BigUint::from_bytes_le(&a_b);
        let premaster_secret = (&a_pub_num * verifier.modpow(&u, &N)).modpow(&b, &N);

        let session_key = WowSRPServer::derive_session_key(
            &premaster_secret
                .to_bytes_le()
                .try_into()
                .expect("correct size"),
        );

        let hn_xor_hg: Vec<_> = Sha1::digest(&N.to_bytes_le())
            .iter()
            .zip(Sha1::digest(&G.to_bytes_le()))
            .map(|(f, s)| f ^ s)
            .collect();

        let server_m = {
            let mut sha = Sha1::new();
            sha.update(&hn_xor_hg);
            sha.update(&self.identity_hash);
            sha.update(&self.salt.0);
            sha.update(a_pub);
            sha.update(&self.b_pub);
            sha.update(&session_key);
            sha.finalize()
        };

        println!("A: {}", a_pub_num);
        println!("I: {:02X?}", self.identity_hash);
        println!("I: {:?}", self.identity_hash);
        println!("v: {}", verifier);
        println!("b: {}", b);
        println!("B: {:?}", self.b_pub);
        println!("u: {}", u);
        println!("S: {:?}", premaster_secret.to_bytes_be());
        println!("K: {:?}", session_key);
        println!("s: {:?}", &self.salt.0);
        println!("hash: {:?}", hn_xor_hg);
        println!("M1: {:?}", server_m);
        println!("M2: {:?}", client_m);

        if server_m.as_slice() == client_m {
            Some(session_key)
        } else {
            None
        }
    }

    /// Calculate the server's public key from a combination of the private key
    /// and the verifier.
    fn calculate_b_pub(b: &[u8; 32], v: &Verifier) -> [u8; 32] {
        let fst = G.modpow(&BigUint::from_bytes_be(b), &N);
        let snd = BigUint::from(v) * BigUint::from(3u8);
        let b_pub = ((fst + snd) % &*N)
            .to_bytes_le()
            .try_into()
            .expect("correct size");
        println!("b_pub: {:?}", b_pub);
        println!("b: {:?}", b);
        println!("v: {:?}", v.0);
        b_pub
    }

    /// Calculates the session key by running it through a SHA1 interleave.
    fn derive_session_key(premaster_secret: &[u8; 32]) -> [u8; 40] {
        let mut left = [0u8; 16];
        let mut right = [0u8; 16];
        for (i, split) in premaster_secret.chunks(2).enumerate() {
            left[i] = split[0];
            right[i] = split[1];
        }

        println!("left: {:?}", left);
        println!("right: {:?}", right);

        let start = premaster_secret
            .iter()
            .enumerate()
            .find(|(_, &v)| v != 0)
            .map(|(i, &v)| if v == 0 { i } else { i + 1 })
            .unwrap_or(premaster_secret.len())
            / 2;

        println!("start: {}", start);

        let left = Sha1::digest(&left[start..]);
        let right = Sha1::digest(&right[start..]);

        println!("left: {:?}", left);
        println!("right: {:?}", right);

        let mut k = [0u8; 40];
        for (i, original) in k.chunks_mut(2).enumerate() {
            original[0] = left[i];
            original[1] = right[i];
        }
        k
    }
}

#[cfg(test)]
mod test {
    use crate::{Salt, Verifier, WowSRPServer};

    #[test]
    pub fn test_session_key_derivation() {
        let s: [u8; 32] = [
            19, 10, 81, 2, 224, 175, 69, 69, 84, 172, 123, 122, 83, 70, 70, 11, 104, 26, 227, 161,
            13, 124, 152, 156, 116, 130, 69, 161, 134, 49, 47, 87,
        ];

        let K = WowSRPServer::derive_session_key(&s);

        let K_expected: [u8; 40] = [
            250, 249, 162, 120, 246, 212, 243, 32, 54, 127, 15, 13, 84, 137, 96, 197, 162, 197, 95,
            221, 107, 218, 252, 23, 37, 95, 250, 83, 182, 53, 105, 254, 23, 14, 207, 191, 85, 207,
            209, 111,
        ];

        assert_eq!(K, K_expected);
    }

    #[test]
    pub fn test_challenge_response() {
        let a_pub: [u8; 32] = [
            161, 6, 45, 226, 95, 140, 75, 203, 143, 102, 171, 182, 96, 203, 237, 67, 17, 103, 16,
            227, 227, 142, 50, 15, 13, 77, 41, 161, 5, 167, 206, 21,
        ];

        let client_m: [u8; 20] = [
            79, 160, 38, 217, 3, 168, 13, 96, 14, 75, 198, 236, 162, 247, 255, 220, 89, 145, 220,
            68,
        ];

        let server = WowSRPServer::new(
            &"ARLYON",
            Salt([
                187, 90, 185, 129, 207, 201, 1, 39, 118, 43, 185, 47, 102, 19, 75, 54, 17, 102,
                255, 182, 144, 248, 239, 202, 238, 158, 71, 164, 216, 195, 53, 226,
            ]),
            Verifier::from_raw([
                44, 42, 171, 164, 129, 208, 59, 156, 50, 148, 246, 223, 12, 222, 85, 21, 129, 251,
                36, 170, 7, 130, 79, 109, 238, 227, 72, 88, 196, 33, 67, 90,
            ]),
        );

        assert!(server
            .verify_challenge_response(&a_pub, &client_m)
            .is_some())
    }

    #[test]
    pub fn test_calculate_b_pub() {
        let b_pub = [
            207, 248, 81, 226, 241, 107, 212, 253, 104, 21, 206, 66, 202, 67, 72, 65, 242, 27, 42,
            111, 204, 187, 209, 246, 130, 204, 13, 78, 184, 205, 74, 56,
        ];

        let b = [
            240, 164, 187, 96, 28, 179, 229, 3, 65, 38, 208, 199, 149, 115, 25, 211, 203, 13, 123,
            214, 254, 46, 60, 159, 111, 12, 39, 40, 23, 85, 118, 31,
        ];

        let v = Verifier::from_raw([
            110, 114, 108, 105, 100, 115, 110, 114, 100, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);

        assert_eq!(WowSRPServer::calculate_b_pub(&b, &v), b_pub)
    }

    #[test]
    pub fn verifier_from_credentials() {
        let salt: [u8; 32] = [
            187, 90, 185, 129, 207, 201, 1, 39, 118, 43, 185, 47, 102, 19, 75, 54, 17, 102, 255,
            182, 144, 248, 239, 202, 238, 158, 71, 164, 216, 195, 53, 226,
        ];

        let v_expected: [u8; 32] = [
            44, 42, 171, 164, 129, 208, 59, 156, 50, 148, 246, 223, 12, 222, 85, 21, 129, 251, 36,
            170, 7, 130, 79, 109, 238, 227, 72, 88, 196, 33, 67, 90,
        ];

        let v = Verifier::from_credentials("ARLYON", "TEST", &Salt(salt));

        assert_eq!(v.0, v_expected);
    }
}
