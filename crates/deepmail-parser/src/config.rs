//! Configuration for deepmail-parser.
//! Env prefix: DEEPMAIL_PARSER_

use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// PostgreSQL connection URL for deepmail_parser database.
    pub database_url: String,

    /// PostgreSQL connection URL for deepmail_ingest database.
    /// Used only for cross-service `job_progress` updates.
    pub ingest_database_url: String,

    /// NATS server URL.
    pub nats_url: String,

    /// S3/MinIO endpoint URL.
    pub s3_endpoint: String,

    /// S3 bucket for quarantined email files (input).
    pub s3_quarantine_bucket: String,

    /// S3 bucket for attachment blobs (output).
    pub s3_attachments_bucket: String,

    /// S3 region.
    #[serde(default = "Config::default_region")]
    pub s3_region: String,

    /// AWS/MinIO access key ID.
    pub s3_access_key_id: String,

    /// AWS/MinIO secret access key.
    pub s3_secret_access_key: String,

    /// Force path-style S3 URLs (required for MinIO).
    #[serde(default = "Config::default_force_path_style")]
    pub s3_force_path_style: bool,

    /// Number of NATS messages to process concurrently.
    #[serde(default = "Config::default_concurrency")]
    pub concurrency: usize,

    /// Seconds to wait before re-delivering a NAKed message.
    #[serde(default = "Config::default_nak_delay_secs")]
    pub nak_delay_secs: u64,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_PARSER")
    }
    fn default_region() -> String { "us-east-1".into() }
    fn default_force_path_style() -> bool { true }
    fn default_concurrency() -> usize { 8 }
    fn default_nak_delay_secs() -> u64 { 5 }
}
