use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ReportConfig {
    pub database_url: String,
    pub parser_database_url: String,
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
    pub scoring_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_s3_endpoint")]
    pub s3_endpoint: String,
    #[serde(default = "default_s3_bucket")]
    pub s3_bucket: String,
    #[serde(default = "default_s3_region")]
    pub s3_region: String,
    #[serde(default)]
    pub aws_access_key_id: String,
    #[serde(default)]
    pub aws_secret_access_key: String,
}

fn default_grpc_port() -> u16 {
    50065
}
fn default_http_port() -> u16 {
    8080
}
fn default_s3_endpoint() -> String {
    "http://localhost:9000".to_string()
}
fn default_s3_bucket() -> String {
    "deepmail-reports".to_string()
}
fn default_s3_region() -> String {
    "us-east-1".to_string()
}

impl ReportConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        deepmail_common::config::load_config::<Self>("DEEPMAIL_REPORT").map_err(Into::into)
    }
}
