/// Configuration for deepmail-body service.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BodyConfig {
    pub database_url: String,
    pub parser_database_url: String,
    pub ingest_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default)]
    pub ml_service_url: String,
    #[serde(default = "default_http_timeout")]
    pub http_timeout_secs: u64,
}

fn default_grpc_port() -> u16 { 50059 }
fn default_http_timeout() -> u64 { 5 }

impl BodyConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        deepmail_common::config::load_config("DEEPMAIL_BODY")
    }
}
