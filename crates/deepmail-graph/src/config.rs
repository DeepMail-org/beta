use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GraphConfig {
    pub database_url: String,
    pub ioc_database_url: String,
    pub scoring_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_neo4j_uri")]
    pub neo4j_uri: String,
    #[serde(default = "default_neo4j_user")]
    pub neo4j_user: String,
    #[serde(default = "default_neo4j_password")]
    pub neo4j_password: String,
    #[serde(default = "default_neo4j_database")]
    pub neo4j_database: String,
    #[serde(default = "default_sync_concurrency")]
    pub sync_concurrency: usize,
    #[serde(default = "default_max_traversal_depth")]
    pub max_traversal_depth: i32,
}

fn default_grpc_port() -> u16 {
    50064
}
fn default_neo4j_uri() -> String {
    "bolt://localhost:7687".to_string()
}
fn default_neo4j_user() -> String {
    "neo4j".to_string()
}
fn default_neo4j_password() -> String {
    "deepmail".to_string()
}
fn default_neo4j_database() -> String {
    "neo4j".to_string()
}
fn default_sync_concurrency() -> usize {
    4
}
fn default_max_traversal_depth() -> i32 {
    4
}

impl GraphConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        deepmail_common::config::load_config::<Self>("DEEPMAIL_GRAPH").map_err(Into::into)
    }
}
