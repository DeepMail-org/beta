//! Configuration for deepmail-header.
//! Env prefix: DEEPMAIL_HEADER_

use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// PostgreSQL URL for deepmail_header database.
    pub database_url: String,

    /// PostgreSQL URL for deepmail_ingest database (job_progress updates).
    pub ingest_database_url: String,

    /// PostgreSQL URL for deepmail_parser database (fetch parsed email).
    pub parser_database_url: String,

    /// gRPC server bind address. Example: 0.0.0.0:50056
    pub grpc_addr: String,

    /// NATS server URL.
    pub nats_url: String,

    /// DNS resolver address. Example: 8.8.8.8:53
    /// Defaults to system resolver if empty.
    #[serde(default)]
    pub dns_resolver: String,

    /// DNS lookup timeout in milliseconds. Default: 5000.
    #[serde(default = "Config::default_dns_timeout_ms")]
    pub dns_timeout_ms: u64,

    /// Number of NATS messages to process concurrently.
    #[serde(default = "Config::default_concurrency")]
    pub concurrency: usize,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_HEADER")
    }
    fn default_dns_timeout_ms() -> u64 { 5000 }
    fn default_concurrency() -> usize { 8 }
}
