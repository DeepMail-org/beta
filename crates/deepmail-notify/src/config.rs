use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct NotifyConfig {
    pub database_url: String,
    pub auth_database_url: String,
    pub tenant_database_url: String,
    pub report_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_auth_grpc_url")]
    pub auth_grpc_url: String,
    #[serde(default)]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default)]
    pub smtp_user: String,
    #[serde(default)]
    pub smtp_password: String,
    #[serde(default = "default_smtp_from")]
    pub smtp_from: String,
    #[serde(default = "default_dashboard_url")]
    pub dashboard_url: String,
    #[serde(default = "default_webhook_timeout")]
    pub webhook_timeout_secs: u64,
    #[serde(default = "default_min_severity")]
    pub min_severity_default: String,
}

fn default_grpc_port() -> u16 {
    50066
}
fn default_http_port() -> u16 {
    8081
}
fn default_auth_grpc_url() -> String {
    "http://localhost:50051".to_string()
}
fn default_smtp_port() -> u16 {
    587
}
fn default_smtp_from() -> String {
    "alerts@deepmail.io".to_string()
}
fn default_dashboard_url() -> String {
    "http://localhost:3000".to_string()
}
fn default_webhook_timeout() -> u64 {
    10
}
fn default_min_severity() -> String {
    "PHISHING".to_string()
}

impl NotifyConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        deepmail_common::config::load_config::<Self>("DEEPMAIL_NOTIFY").map_err(Into::into)
    }
}
