/// Service configuration.

use anyhow::Result;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SandboxFileConfig {
    pub database_url: String,
    pub parser_database_url: String,
    pub ingest_database_url: String,
    pub nats_url: String,
    pub grpc_port: u16,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub tool_timeout_secs: u64,
}

impl SandboxFileConfig {
    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        Ok(Self::from_env())
    }

    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_file".into()),
            parser_database_url: std::env::var("PARSER_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_parser".into()),
            ingest_database_url: std::env::var("INGEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_ingest".into()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".into()),
            grpc_port: std::env::var("GRPC_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(50061),
            s3_endpoint: std::env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".into()),
            s3_bucket: std::env::var("S3_BUCKET")
                .unwrap_or_else(|_| "deepmail-files".into()),
            s3_region: std::env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".into()),
            tool_timeout_secs: std::env::var("TOOL_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}
