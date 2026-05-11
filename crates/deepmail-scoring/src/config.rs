use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ScoringConfig {
    pub database_url: String,
    pub ingest_database_url: String,
    pub header_database_url: String,
    pub dkim_database_url: String,
    pub geo_database_url: String,
    pub ip_database_url: String,
    pub intel_database_url: String,
    pub ioc_database_url: String,
    pub homograph_database_url: String,
    pub body_database_url: String,
    pub sandbox_url_database_url: String,
    pub sandbox_file_database_url: String,
    pub sandbox_dynamic_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    #[serde(default = "default_final_min_signals")]
    pub score_final_min_signals: usize,
    #[serde(default = "default_timeout_minutes")]
    pub score_timeout_minutes: u64,
}

fn default_grpc_port() -> u16 {
    50063
}
fn default_concurrency() -> usize {
    10
}
fn default_final_min_signals() -> usize {
    8
}
fn default_timeout_minutes() -> u64 {
    5
}

impl ScoringConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        deepmail_common::config::load_config::<Self>("DEEPMAIL_SCORING").map_err(Into::into)
    }
}
