//! Refresh token generation and hashing.
//!
//! Refresh tokens are 32 random bytes encoded as 64-char hex strings.
//! Only their SHA-256 hash is stored in the database.

use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a cryptographically random refresh token.
///
/// Returns a 64-character lowercase hex string. Hash with
/// [`hash_refresh_token`] before storing.
pub fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Hash a refresh token with SHA-256 for database storage.
///
/// The raw token is sent to the client. Only the hash is persisted.
pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Hash an API key for database storage. Same scheme as refresh tokens.
pub fn hash_api_key(key: &str) -> String {
    hash_refresh_token(key)
}

/// Generate an API key with prefix.
///
/// Format: `dm_live_{40 random hex chars}`. The full key is shown to the
/// user once; only the prefix and SHA-256 hash are stored in the database.
pub fn generate_api_key() -> (String, String) {
    let mut bytes = [0u8; 20];
    rand::thread_rng().fill_bytes(&mut bytes);
    let random_part = hex::encode(bytes);
    let full_key = format!("dm_live_{random_part}");
    let prefix = full_key[..12].to_string();
    (full_key, prefix)
}
