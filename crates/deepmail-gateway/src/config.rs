pub struct GatewayConfig {
    pub database_url: String,
    pub ingest_database_url: String,
    pub redis_url: String,
    pub http_port: u16,
    pub ingest_http_url: String,
    pub auth_grpc_url: String,
    pub scoring_grpc_url: String,
    pub report_grpc_url: String,
    pub ioc_grpc_url: String,
    pub graph_grpc_url: String,
    pub tenant_grpc_url: String,
    pub billing_grpc_url: String,
    pub notify_grpc_url: String,
    pub rate_limit_per_minute: u32,
    pub graphiql_enabled: bool,
    pub cors_allow_origin: String,
    pub max_upload_bytes: usize,
}

impl GatewayConfig {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            database_url: require_env("DATABASE_URL")?,
            ingest_database_url: require_env("INGEST_DATABASE_URL")?,
            redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
            http_port: parse_env("HTTP_PORT", 3001),
            ingest_http_url: env_or("INGEST_HTTP_URL", "http://127.0.0.1:8090"),
            auth_grpc_url: env_or("AUTH_GRPC_URL", "http://127.0.0.1:50051"),
            scoring_grpc_url: env_or("SCORING_GRPC_URL", "http://127.0.0.1:50063"),
            report_grpc_url: env_or("REPORT_GRPC_URL", "http://127.0.0.1:50065"),
            ioc_grpc_url: env_or("IOC_GRPC_URL", "http://127.0.0.1:50057"),
            graph_grpc_url: env_or("GRAPH_GRPC_URL", "http://127.0.0.1:50064"),
            tenant_grpc_url: env_or("TENANT_GRPC_URL", "http://127.0.0.1:50052"),
            billing_grpc_url: env_or("BILLING_GRPC_URL", "http://127.0.0.1:50067"),
            notify_grpc_url: env_or("NOTIFY_GRPC_URL", "http://127.0.0.1:50066"),
            rate_limit_per_minute: parse_env("RATE_LIMIT_PER_MINUTE", 100),
            graphiql_enabled: env_or("GRAPHIQL_ENABLED", "false") == "true",
            cors_allow_origin: env_or("CORS_ALLOW_ORIGIN", "*"),
            max_upload_bytes: parse_env("MAX_UPLOAD_MB", 25usize) * 1024 * 1024,
        })
    }
}

fn require_env(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("required env var {key} is not set"))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_env<T: std::str::FromStr + Copy>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| {
            v.parse().ok().or_else(|| {
                tracing::warn!(key, value = %v, "invalid env var value, using default");
                None
            })
        })
        .unwrap_or(default)
}
