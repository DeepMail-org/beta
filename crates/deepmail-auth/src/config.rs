//! Configuration for deepmail-auth.
//!
//! All values are read from environment variables prefixed with
//! `DEEPMAIL_AUTH_` (e.g. `DEEPMAIL_AUTH_DATABASE_URL`).
//! A `config.toml` file in the working directory is also read,
//! with environment variables taking precedence.

use deepmail_common::config::load_config;

/// Runtime configuration for the deepmail-auth service.
#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// PostgreSQL connection URL.
    /// Example: `postgres://user:pass@localhost:5432/deepmail_auth`.
    pub database_url: String,

    /// gRPC server bind address.
    /// Example: `0.0.0.0:50051`.
    pub grpc_addr: String,

    /// NATS server URL.
    /// Example: `nats://localhost:4222`.
    pub nats_url: String,

    /// RS256 private key in PEM format (for signing JWTs).
    /// Load from Vault in production. Never commit to git.
    pub jwt_private_key_pem: String,

    /// RS256 public key in PEM format (for verifying JWTs).
    pub jwt_public_key_pem: String,

    /// Access token TTL in seconds. Default: 900 (15 minutes).
    #[serde(default = "Config::default_access_expiry")]
    pub jwt_access_expiry_seconds: u64,

    /// Refresh token TTL in days. Default: 7.
    #[serde(default = "Config::default_refresh_expiry_days")]
    pub jwt_refresh_expiry_days: u64,

    /// OTP code TTL in seconds. Default: 600 (10 minutes).
    #[serde(default = "Config::default_otp_expiry")]
    pub otp_expiry_seconds: u64,

    /// Maximum OTP verification attempts before the code is invalidated.
    /// Default: 5.
    #[serde(default = "Config::default_otp_max_attempts")]
    pub otp_max_attempts: i32,

    /// How long (in minutes) an IP is locked after exceeding failed login limit.
    /// Default: 15.
    #[serde(default = "Config::default_lockout_minutes")]
    pub lockout_duration_minutes: u64,

    /// Failed login attempts before IP lockout triggers. Default: 5.
    #[serde(default = "Config::default_lockout_max_attempts")]
    pub lockout_max_attempts: i32,

    /// gRPC address of deepmail-otp-smtp for sending OTP emails.
    /// Example: `http://deepmail-otp-smtp:50052`.
    pub smtp_grpc_addr: String,
}

impl Config {
    /// Load config from environment and optional `config.toml`.
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_AUTH")
    }

    fn default_access_expiry() -> u64 {
        900
    }
    fn default_refresh_expiry_days() -> u64 {
        7
    }
    fn default_otp_expiry() -> u64 {
        600
    }
    fn default_otp_max_attempts() -> i32 {
        5
    }
    fn default_lockout_minutes() -> u64 {
        15
    }
    fn default_lockout_max_attempts() -> i32 {
        5
    }
}
