use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct IntelConfig {
    pub database_url: String,
    pub parser_database_url: String,
    pub ingest_database_url: String,
    pub nats_url: String,
    pub redis_url: String,
    #[serde(default = "IntelConfig::default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default)]
    pub virustotal_api_key: String,
    #[serde(default)]
    pub abuseipdb_api_key: String,
    #[serde(default)]
    pub greynoise_api_key: String,
    #[serde(default)]
    pub ipinfo_token: String,
    #[serde(default)]
    pub shodan_api_key: String,
    #[serde(default)]
    pub otx_api_key: String,
    #[serde(default = "IntelConfig::default_http_timeout")]
    pub http_timeout_secs: u64,
    #[serde(default = "IntelConfig::default_vt_rate")]
    pub vt_rate_limit_per_min: u32,
    #[serde(default = "IntelConfig::default_cache_cleanup")]
    pub cache_cleanup_hours: u64,
    #[serde(default = "IntelConfig::default_telemetry_flush")]
    pub telemetry_flush_secs: u64,
}

impl IntelConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_INTEL")
    }
    fn default_grpc_port() -> u16 { 50056 }
    fn default_http_timeout() -> u64 { 10 }
    fn default_vt_rate() -> u32 { 4 }
    fn default_cache_cleanup() -> u64 { 1 }
    fn default_telemetry_flush() -> u64 { 300 }
}
