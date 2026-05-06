//! Async DNS lookup helpers.
//!
//! Wraps hickory-resolver 0.24 with:
//!   - Configurable timeout
//!   - Structured result including latency
//!   - All answers returned as Vec<String>

use std::time::Instant;

/// Result of a single DNS lookup.
#[derive(Debug, Clone)]
pub struct DnsResult {
    pub queried_name: String,
    pub record_type: String,
    pub answers: Vec<String>,
    pub ttl_seconds: Option<i32>,
    pub nxdomain: bool,
    pub error: Option<String>,
    pub latency_ms: i32,
}

/// DNS resolver wrapper.
/// Created once at startup and shared via Arc.
pub struct Resolver {
    pub(crate) inner: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    /// Create a resolver using the system configuration.
    pub fn new_system() -> Result<Self, crate::error::HeaderError> {
        use hickory_resolver::{
            config::{ResolverConfig, ResolverOpts},
            TokioAsyncResolver,
        };
        let resolver = TokioAsyncResolver::tokio(
            ResolverConfig::default(),
            ResolverOpts::default(),
        );
        Ok(Self { inner: resolver })
    }

    /// Create a resolver pointing at a specific server (e.g. 8.8.8.8:53).
    pub fn new_with_server(addr: &str) -> Result<Self, crate::error::HeaderError> {
        use hickory_resolver::{
            config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
            TokioAsyncResolver,
        };

        let socket_addr: std::net::SocketAddr = addr
            .parse()
            .map_err(|e| crate::error::HeaderError::Dns(format!("invalid DNS addr: {e}")))?;

        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr,
            protocol: Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: true,
            bind_addr: None,
        });

        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());
        Ok(Self { inner: resolver })
    }

    /// Perform a TXT record lookup. Returns all TXT strings for the name.
    pub async fn lookup_txt(&self, name: &str) -> DnsResult {
        let start = Instant::now();
        match self.inner.txt_lookup(name).await {
            Ok(lookup) => {
                let answers: Vec<String> = lookup
                    .iter()
                    .map(|txt| txt.to_string())
                    .collect();
                let ttl = lookup
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map(|r| r.ttl() as i32);
                DnsResult {
                    queried_name: name.to_string(),
                    record_type: "TXT".to_string(),
                    answers,
                    ttl_seconds: ttl,
                    nxdomain: false,
                    error: None,
                    latency_ms: start.elapsed().as_millis() as i32,
                }
            }
            Err(e) => {
                let is_no_records = matches!(
                    e.kind(),
                    hickory_resolver::error::ResolveErrorKind::NoRecordsFound { .. }
                );
                DnsResult {
                    queried_name: name.to_string(),
                    record_type: "TXT".to_string(),
                    answers: vec![],
                    ttl_seconds: None,
                    nxdomain: is_no_records,
                    error: Some(e.to_string()),
                    latency_ms: start.elapsed().as_millis() as i32,
                }
            }
        }
    }

    /// Perform an MX record lookup.
    pub async fn lookup_mx(&self, name: &str) -> DnsResult {
        let start = Instant::now();
        match self.inner.mx_lookup(name).await {
            Ok(lookup) => {
                let answers: Vec<String> = lookup
                    .iter()
                    .map(|mx| mx.exchange().to_string())
                    .collect();
                DnsResult {
                    queried_name: name.to_string(),
                    record_type: "MX".to_string(),
                    answers,
                    ttl_seconds: None,
                    nxdomain: false,
                    error: None,
                    latency_ms: start.elapsed().as_millis() as i32,
                }
            }
            Err(e) => DnsResult {
                queried_name: name.to_string(),
                record_type: "MX".to_string(),
                answers: vec![],
                ttl_seconds: None,
                nxdomain: matches!(
                    e.kind(),
                    hickory_resolver::error::ResolveErrorKind::NoRecordsFound { .. }
                ),
                error: Some(e.to_string()),
                latency_ms: start.elapsed().as_millis() as i32,
            },
        }
    }

    /// Perform a PTR (reverse DNS) lookup for an IP.
    pub async fn lookup_ptr(&self, ip: std::net::IpAddr) -> DnsResult {
        let start = Instant::now();
        match self.inner.reverse_lookup(ip).await {
            Ok(lookup) => {
                let answers: Vec<String> =
                    lookup.iter().map(|name| name.to_string()).collect();
                DnsResult {
                    queried_name: ip.to_string(),
                    record_type: "PTR".to_string(),
                    answers,
                    ttl_seconds: None,
                    nxdomain: false,
                    error: None,
                    latency_ms: start.elapsed().as_millis() as i32,
                }
            }
            Err(e) => DnsResult {
                queried_name: ip.to_string(),
                record_type: "PTR".to_string(),
                answers: vec![],
                ttl_seconds: None,
                nxdomain: true,
                error: Some(e.to_string()),
                latency_ms: start.elapsed().as_millis() as i32,
            },
        }
    }
}
