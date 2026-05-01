//! Password hashing and verification using Argon2id.
//!
//! Parameters: time_cost=3, memory=65536 KiB, parallelism=4.
//! These match OWASP recommendations for interactive logins.

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2, Params, Version,
};
use rand::rngs::OsRng;

use crate::error::AuthError;

/// Hash a plaintext password with Argon2id. CPU-bound.
fn hash_password_sync(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let params = Params::new(65536, 3, 4, None)
        .map_err(|e| AuthError::Argon2(e.to_string()))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::Argon2(e.to_string()))?
        .to_string();
    Ok(hash)
}

/// Hash a plaintext password asynchronously.
///
/// Offloads CPU work to a blocking thread pool. Never call this from
/// inside another `tokio::task::spawn_blocking` closure.
pub async fn hash_password(password: &str) -> Result<String, AuthError> {
    let password = password.to_string();
    tokio::task::spawn_blocking(move || hash_password_sync(&password))
        .await
        .map_err(|e| AuthError::Internal(format!("spawn_blocking join error: {e}")))?
}

/// Verify a plaintext password against an Argon2id PHC hash string. CPU-bound.
fn verify_password_sync(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| AuthError::Argon2(e.to_string()))?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AuthError::Argon2(e.to_string())),
    }
}

/// Verify a plaintext password asynchronously.
pub async fn verify_password(
    password: &str,
    hash: &str,
) -> Result<bool, AuthError> {
    let password = password.to_string();
    let hash = hash.to_string();
    tokio::task::spawn_blocking(move || verify_password_sync(&password, &hash))
        .await
        .map_err(|e| AuthError::Internal(format!("spawn_blocking join error: {e}")))?
}
