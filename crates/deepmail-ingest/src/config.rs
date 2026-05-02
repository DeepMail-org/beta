//! Configuration for deepmail-ingest.
//! Env prefix: DEEPMAIL_INGEST_

use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// PostgreSQL connection URL for deepmail_ingest database.
    pub database_url: String,

    /// HTTP server bind address.
    /// Example: 0.0.0.0:8082
    pub http_addr: String,

    /// NATS server URL.
    pub nats_url: String,

    /// S3/MinIO endpoint URL.
    /// Example: http://localhost:9000  (MinIO local)
    /// Example: https://s3.amazonaws.com  (AWS S3)
    pub s3_endpoint: String,

    /// S3 bucket name for quarantined email files.
    pub s3_bucket: String,

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

    /// Maximum file upload size in megabytes.
    /// Default: 25 MB. Per-tenant limits can be lower (checked at validation).
    #[serde(default = "Config::default_max_upload_mb")]
    pub max_upload_size_mb: u64,

    /// Address of deepmail-tenant gRPC for fetching per-tenant settings.
    pub tenant_grpc_addr: String,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_INGEST")
    }
    fn default_region() -> String { "us-east-1".into() }
    fn default_force_path_style() -> bool { true }
    fn default_max_upload_mb() -> u64 { 25 }
}
