//! Configuration for deepmail-tenant.
//! Env prefix: DEEPMAIL_TENANT_

use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// PostgreSQL connection URL for deepmail_tenant database.
    pub database_url: String,

    /// gRPC server bind address. Example: 0.0.0.0:50052
    pub grpc_addr: String,

    /// HTTP server bind address for Razorpay webhook receiver.
    /// Example: 0.0.0.0:8081
    pub http_addr: String,

    /// NATS server URL. Example: nats://localhost:4222
    pub nats_url: String,

    /// Razorpay webhook secret for HMAC-SHA256 signature verification.
    /// Set this in Vault. Never commit.
    pub razorpay_webhook_secret: String,

    /// Razorpay API key ID for making API calls (optional at startup).
    pub razorpay_key_id: Option<String>,

    /// Razorpay API key secret (optional at startup).
    pub razorpay_key_secret: Option<String>,

    /// Default retention period in days for new tenants.
    #[serde(default = "Config::default_retention_days")]
    pub default_retention_days: i32,

    /// Invite token TTL in hours. Default: 48.
    #[serde(default = "Config::default_invite_ttl_hours")]
    pub invite_ttl_hours: u64,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_TENANT")
    }
    fn default_retention_days() -> i32 { 90 }
    fn default_invite_ttl_hours() -> u64 { 48 }
}
