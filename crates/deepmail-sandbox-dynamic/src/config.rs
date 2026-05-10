/// Service configuration.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DynamicConfig {
    pub database_url: String,
    pub sandbox_file_database_url: String,
    pub ingest_database_url: String,
    pub nats_url: String,
    pub grpc_port: u16,
    pub cape_api_url: String,
    pub cape_api_token: String,
    pub cape_poll_interval_secs: u64,
    pub cape_timeout_secs: u64,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
}

impl DynamicConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_dynamic".into()),
            sandbox_file_database_url: std::env::var("SANDBOX_FILE_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_file".into()),
            ingest_database_url: std::env::var("INGEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ingest".into()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".into()),
            grpc_port: std::env::var("GRPC_PORT")
                .ok().and_then(|p| p.parse().ok())
                .unwrap_or(50062),
            cape_api_url: std::env::var("CAPE_API_URL").unwrap_or_default(),
            cape_api_token: std::env::var("CAPE_API_TOKEN").unwrap_or_default(),
            cape_poll_interval_secs: std::env::var("CAPE_POLL_INTERVAL_SECS")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(10),
            cape_timeout_secs: std::env::var("CAPE_TIMEOUT_SECS")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(180),
            s3_endpoint: std::env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".into()),
            s3_bucket: std::env::var("S3_BUCKET")
                .unwrap_or_else(|_| "deepmail-dynamic".into()),
            s3_region: std::env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".into()),
        }
    }
}
