use deepmail_common::config::load_config;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    pub grpc_addr: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    #[serde(default)]
    pub smtp_starttls: bool,
    #[serde(default = "Config::default_smtp_enabled")]
    pub smtp_enabled: bool,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        load_config::<Self>("DEEPMAIL_OTP_SMTP")
    }

    fn default_smtp_enabled() -> bool {
        true
    }
}
