//! OTP generation and verification.
//!
//! OTPs are 8 uppercase alphanumeric characters (A-Z, 0-9).
//! The plaintext code is sent by email. Only its Argon2id hash
//! is stored in the database — never the plaintext.

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::{distributions::Alphanumeric, rngs::OsRng, Rng};

use crate::error::AuthError;

/// Generate a cryptographically random 8-character alphanumeric OTP.
///
/// Uses `OsRng` for OS-level entropy.
/// Output characters are drawn from A-Z + 0-9 (case-normalized to uppercase).
/// Example output: "A3KZ9M2P".
pub fn generate_otp() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// Hash a plaintext OTP for storage. CPU-bound.
fn hash_otp_sync(otp: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(otp.as_bytes(), &salt)
        .map_err(|e| AuthError::Argon2(e.to_string()))?
        .to_string();
    Ok(hash)
}

/// Hash a plaintext OTP asynchronously for storage.
pub async fn hash_otp(otp: &str) -> Result<String, AuthError> {
    let otp = otp.to_string();
    tokio::task::spawn_blocking(move || hash_otp_sync(&otp))
        .await
        .map_err(|e| AuthError::Internal(format!("spawn_blocking join error: {e}")))?
}

/// Verify a plaintext OTP against its stored Argon2id hash.
/// Constant-time comparison.
fn verify_otp_sync(otp: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed =
        PasswordHash::new(hash).map_err(|e| AuthError::Argon2(e.to_string()))?;
    match Argon2::default().verify_password(otp.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AuthError::Argon2(e.to_string())),
    }
}

/// Verify a plaintext OTP asynchronously.
pub async fn verify_otp(otp: &str, hash: &str) -> Result<bool, AuthError> {
    let otp = otp.to_string();
    let hash = hash.to_string();
    tokio::task::spawn_blocking(move || verify_otp_sync(&otp, &hash))
        .await
        .map_err(|e| AuthError::Internal(format!("spawn_blocking join error: {e}")))?
}
