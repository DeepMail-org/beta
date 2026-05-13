use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BillingConfig {
    pub database_url: String,
    pub auth_database_url: String,
    pub tenant_database_url: String,
    pub nats_url: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default)]
    pub razorpay_key_id: String,
    #[serde(default)]
    pub razorpay_key_secret: String,
    #[serde(default)]
    pub razorpay_webhook_secret: String,
}

fn default_grpc_port() -> u16 {
    50067
}
fn default_http_port() -> u16 {
    8082
}

impl BillingConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        deepmail_common::config::load_config::<Self>("DEEPMAIL_BILLING").map_err(Into::into)
    }
}
