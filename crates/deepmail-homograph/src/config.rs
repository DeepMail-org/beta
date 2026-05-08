/// Configuration for deepmail-homograph service.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HomographConfig {
    pub database_url: String,
    pub parser_database_url: String,
    pub ioc_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_min_score_threshold")]
    pub min_score_threshold: f32,
}

fn default_grpc_port() -> u16 { 50058 }
fn default_min_score_threshold() -> f32 { 0.30 }

impl HomographConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        deepmail_common::config::load_config("DEEPMAIL_HOMOGRAPH")
    }
}
