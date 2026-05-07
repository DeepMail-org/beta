use deepmail_common::config::load_config;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct GeoConfig {
    pub database_url: String,
    pub ingest_database_url: String,
    pub parser_database_url: String,
    pub nats_url: String,
    pub redis_url: String,
    #[serde(default = "GeoConfig::default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "GeoConfig::default_city_db")]
    pub geoip_city_db_path: String,
    #[serde(default = "GeoConfig::default_asn_db")]
    pub geoip_asn_db_path: String,
    #[serde(default)]
    pub ipinfo_token: String,
    #[serde(default)]
    pub abuseipdb_api_key: String,
    #[serde(default)]
    pub greynoise_api_key: String,
    #[serde(default = "GeoConfig::default_whois_timeout")]
    pub whois_timeout_secs: u64,
    #[serde(default = "GeoConfig::default_http_timeout")]
    pub http_timeout_secs: u64,
    #[serde(default = "GeoConfig::default_cache_ttl")]
    pub enrichment_cache_ttl: u64,
    #[serde(default = "GeoConfig::default_concurrency")]
    pub concurrency: usize,
}

impl GeoConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_GEO")
    }
    fn default_grpc_port() -> u16 { 50054 }
    fn default_city_db() -> String { "/etc/deepmail/GeoLite2-City.mmdb".to_string() }
    fn default_asn_db() -> String { "/etc/deepmail/GeoLite2-ASN.mmdb".to_string() }
    fn default_whois_timeout() -> u64 { 5 }
    fn default_http_timeout() -> u64 { 3 }
    fn default_cache_ttl() -> u64 { 3600 }
    fn default_concurrency() -> usize { 6 }
}
