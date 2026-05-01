//! Error types for deepmail-auth.

use thiserror::Error;

/// All errors that can occur in the auth service.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Database (PostgreSQL / sqlx) failure.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Requested user does not exist.
    #[error("user not found")]
    UserNotFound,

    /// Login credentials are wrong (deliberately vague).
    #[error("invalid credentials")]
    InvalidCredentials,

    /// IP address is locked out for further login attempts.
    #[error("account is locked until {0}")]
    AccountLocked(chrono::DateTime<chrono::Utc>),

    /// Account exists but has not yet verified email / been activated.
    #[error("account is not active — verify your email first")]
    AccountNotActive,

    /// Account is administratively flagged (e.g. abuse).
    #[error("account is flagged: {0}")]
    AccountFlagged(String),

    /// OTP code's `expires_at` is in the past.
    #[error("OTP has expired")]
    OtpExpired,

    /// OTP code's plaintext does not match the stored hash.
    #[error("OTP is invalid")]
    OtpInvalid,

    /// User exhausted the per-OTP retry budget.
    #[error("OTP max attempts exceeded")]
    OtpMaxAttempts,

    /// No active OTP exists for the requested user/type pair.
    #[error("OTP not found")]
    OtpNotFound,

    /// JWT signature/format is invalid.
    #[error("token is invalid")]
    TokenInvalid,

    /// JWT `exp` claim is in the past.
    #[error("token has expired")]
    TokenExpired,

    /// Refresh token does not exist or has already been revoked.
    #[error("refresh token not found or already revoked")]
    RefreshTokenInvalid,

    /// Generic password hashing failure.
    #[error("password hashing error: {0}")]
    PasswordHash(String),

    /// JWT-library failure (signing/verification).
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// Argon2id hashing/verification failure.
    #[error("argon2 error: {0}")]
    Argon2(String),

    /// Catch-all for unexpected internal failures.
    #[error("internal error: {0}")]
    Internal(String),

    /// Registration: email or username already taken.
    #[error("user already exists with that email or username")]
    UserAlreadyExists,

    /// Input failed validation (bad format, missing field, …).
    #[error("input validation failed: {0}")]
    Validation(String),
}

impl From<AuthError> for tonic::Status {
    fn from(e: AuthError) -> Self {
        match e {
            AuthError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in auth service");
                tonic::Status::internal("internal server error")
            }
            AuthError::UserNotFound => tonic::Status::not_found("user not found"),
            AuthError::InvalidCredentials => {
                // Deliberately vague to prevent user enumeration.
                tonic::Status::unauthenticated("invalid credentials")
            }
            AuthError::AccountLocked(until) => tonic::Status::resource_exhausted(
                format!("account locked until {until}"),
            ),
            AuthError::AccountNotActive => tonic::Status::failed_precondition(
                "account not active — verify your email",
            ),
            AuthError::AccountFlagged(reason) => {
                tonic::Status::permission_denied(format!("account flagged: {reason}"))
            }
            AuthError::OtpExpired => {
                tonic::Status::deadline_exceeded("OTP has expired")
            }
            AuthError::OtpInvalid => tonic::Status::unauthenticated("invalid OTP"),
            AuthError::OtpMaxAttempts => tonic::Status::resource_exhausted(
                "too many OTP attempts — request a new code",
            ),
            AuthError::OtpNotFound => {
                tonic::Status::not_found("no active OTP found")
            }
            AuthError::TokenInvalid | AuthError::TokenExpired => {
                tonic::Status::unauthenticated("token is invalid or expired")
            }
            AuthError::RefreshTokenInvalid => {
                tonic::Status::unauthenticated("refresh token is invalid")
            }
            AuthError::PasswordHash(ref msg)
            | AuthError::Argon2(ref msg)
            | AuthError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in auth service");
                tonic::Status::internal("internal server error")
            }
            AuthError::Jwt(ref inner) => {
                tracing::warn!(error = %inner, "JWT error");
                tonic::Status::unauthenticated("token is invalid")
            }
            AuthError::UserAlreadyExists => tonic::Status::already_exists(
                "a user with that email or username already exists",
            ),
            AuthError::Validation(ref msg) => {
                tonic::Status::invalid_argument(msg.clone())
            }
        }
    }
}
