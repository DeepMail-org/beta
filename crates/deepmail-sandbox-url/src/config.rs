/// Configuration for deepmail-sandbox-url.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SandboxUrlConfig {
    pub database_url: String,
    pub nats_url: String,
    pub grpc_port: u16,
    pub docker_sock: String,
    pub playwright_image: String,
    pub container_timeout_secs: u64,
    pub sandbox_concurrency: usize,
    pub s3_endpoint: String,
    pub s3_bucket: String,
    pub s3_region: String,
}

impl SandboxUrlConfig {
    pub fn load() -> anyhow::Result<Self> {
        let cfg = deepmail_common::config::load_config::<Self>("SANDBOX_URL")
            .map_err(|e| anyhow::anyhow!("config: {}", e))?;
        Ok(cfg)
    }

    /// Fallback from environment variables with defaults.
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://deepmail:deepmailpw@localhost:5432/deepmail_sandbox_url".into()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".into()),
            grpc_port: std::env::var("GRPC_PORT")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(50060),
            docker_sock: std::env::var("DOCKER_SOCK")
                .unwrap_or_else(|_| "/var/run/docker.sock".into()),
            playwright_image: std::env::var("PLAYWRIGHT_IMAGE")
                .unwrap_or_else(|_| "mcr.microsoft.com/playwright:v1.40.0-jammy".into()),
            container_timeout_secs: std::env::var("CONTAINER_TIMEOUT_SECS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(30),
            sandbox_concurrency: std::env::var("SANDBOX_CONCURRENCY")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(3),
            s3_endpoint: std::env::var("S3_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".into()),
            s3_bucket: std::env::var("S3_BUCKET")
                .unwrap_or_else(|_| "deepmail-sandbox".into()),
            s3_region: std::env::var("S3_REGION")
                .unwrap_or_else(|_| "us-east-1".into()),
        }
    }
}
