use std::time::Instant;

use crate::error::DkimError;

#[derive(Debug, Clone)]
pub struct DnsKeyResult {
    pub selector: String,
    pub domain: String,
    pub raw_txt: Option<String>,
    pub ttl_seconds: Option<i32>,
    pub found: bool,
    pub error: Option<String>,
    pub latency_ms: i32,
}

pub struct Resolver {
    pub(crate) inner: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    pub fn new_system() -> Result<Self, DkimError> {
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

    pub fn new_with_server(addr: &str) -> Result<Self, DkimError> {
        use hickory_resolver::{
            config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
            TokioAsyncResolver,
        };

        let socket_addr: std::net::SocketAddr = addr
            .parse()
            .map_err(|e| DkimError::Dns(format!("invalid DNS addr: {e}")))?;

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

    pub async fn lookup_dkim_key(&self, selector: &str, domain: &str) -> DnsKeyResult {
        let name = format!("{selector}._domainkey.{domain}");
        let start = Instant::now();

        match self.inner.txt_lookup(&name).await {
            Ok(lookup) => {
                let combined: String = lookup
                    .iter()
                    .map(|txt| txt.to_string())
                    .collect::<Vec<_>>()
                    .join("");
                let ttl = lookup
                    .as_lookup()
                    .record_iter()
                    .next()
                    .map(|r| r.ttl() as i32);
                let found = !combined.is_empty();
                DnsKeyResult {
                    selector: selector.to_string(),
                    domain: domain.to_string(),
                    raw_txt: if found { Some(combined) } else { None },
                    ttl_seconds: ttl,
                    found,
                    error: None,
                    latency_ms: start.elapsed().as_millis() as i32,
                }
            }
            Err(e) => {
                let is_nxdomain = matches!(
                    e.kind(),
                    hickory_resolver::error::ResolveErrorKind::NoRecordsFound { .. }
                );
                DnsKeyResult {
                    selector: selector.to_string(),
                    domain: domain.to_string(),
                    raw_txt: None,
                    ttl_seconds: None,
                    found: false,
                    error: if is_nxdomain { None } else { Some(e.to_string()) },
                    latency_ms: start.elapsed().as_millis() as i32,
                }
            }
        }
    }
}
