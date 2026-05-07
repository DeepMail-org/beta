use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DkimConfig {
    pub database_url: String,
    pub ingest_database_url: String,
    pub grpc_addr: String,
    pub nats_url: String,
    #[serde(default)]
    pub dns_resolver: String,
    #[serde(default = "DkimConfig::default_dns_timeout_ms")]
    pub dns_timeout_ms: u64,
    #[serde(default = "DkimConfig::default_concurrency")]
    pub concurrency: usize,
}

impl DkimConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_DKIM")
    }
    fn default_dns_timeout_ms() -> u64 { 5000 }
    fn default_concurrency() -> usize { 8 }
}
