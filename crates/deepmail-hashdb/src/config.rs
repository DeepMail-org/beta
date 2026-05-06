//! Configuration for deepmail-hashdb.
//! Env prefix: DEEPMAIL_HASHDB_

use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// PostgreSQL connection URL for deepmail_hashdb database.
    pub database_url: String,

    /// gRPC server bind address.
    /// Example: 0.0.0.0:50055
    pub grpc_addr: String,

    /// Redis URL for bloom filter.
    /// Example: redis://localhost:6379
    pub redis_url: String,

    /// Bloom filter Redis key.
    #[serde(default = "Config::default_bloom_key")]
    pub bloom_filter_key: String,

    /// Fallback bloom TTL in seconds when RedisBloom unavailable.
    /// Default: 2592000 (30 days).
    #[serde(default = "Config::default_bloom_ttl")]
    pub bloom_fallback_ttl_seconds: u64,

    /// Max recent hashes to compare against during ssdeep clustering.
    /// Default: 1000.
    #[serde(default = "Config::default_cluster_limit")]
    pub ssdeep_cluster_limit: i64,

    /// ssdeep similarity threshold for clustering. Default: 70.
    #[serde(default = "Config::default_ssdeep_threshold")]
    pub ssdeep_threshold: u32,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_HASHDB")
    }
    fn default_bloom_key() -> String { "deepmail:hashdb:bloom".into() }
    fn default_bloom_ttl() -> u64 { 2_592_000 }
    fn default_cluster_limit() -> i64 { 1000 }
    fn default_ssdeep_threshold() -> u32 { 70 }
}
