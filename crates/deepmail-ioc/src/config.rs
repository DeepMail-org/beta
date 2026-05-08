/// Configuration for deepmail-ioc service.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IocConfig {
    pub database_url: String,
    pub parser_database_url: String,
    pub ingest_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_intel_grpc_url")]
    pub intel_grpc_url: String,
    #[serde(default = "default_enrich_concurrency")]
    pub enrich_concurrency: usize,
    #[serde(default = "default_campaign_window_days")]
    pub campaign_window_days: i64,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

fn default_grpc_port() -> u16 { 50057 }
fn default_intel_grpc_url() -> String { "http://127.0.0.1:50056".into() }
fn default_enrich_concurrency() -> usize { 10 }
fn default_campaign_window_days() -> i64 { 30 }
fn default_similarity_threshold() -> f32 { 0.3 }

impl IocConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        deepmail_common::config::load_config("DEEPMAIL_IOC")
    }
}
