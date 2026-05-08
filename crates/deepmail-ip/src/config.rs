use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct IpConfig {
    pub database_url: String,
    pub parser_database_url: String,
    pub ingest_database_url: String,
    pub nats_url: String,
    pub redis_url: String,
    #[serde(default = "IpConfig::default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default)]
    pub shodan_api_key: String,
    #[serde(default)]
    pub abuseipdb_api_key: String,
    #[serde(default = "IpConfig::default_http_timeout")]
    pub http_timeout_secs: u64,
    #[serde(default = "IpConfig::default_feed_refresh")]
    pub feed_refresh_hours: u64,
    #[serde(default = "IpConfig::default_concurrency")]
    pub concurrency: usize,
}

impl IpConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_IP")
    }
    fn default_grpc_port() -> u16 { 50055 }
    fn default_http_timeout() -> u64 { 10 }
    fn default_feed_refresh() -> u64 { 4 }
    fn default_concurrency() -> usize { 8 }
}
