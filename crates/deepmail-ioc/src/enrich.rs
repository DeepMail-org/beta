/// gRPC client for deepmail-intel enrichment + threat level assignment.

use std::collections::HashMap;

use tonic::transport::Channel;

use deepmail_common::proto::intel::intel_enricher_client::IntelEnricherClient;
use deepmail_common::proto::intel::EnrichIocRequest;

use crate::error::IocError;

/// Threat levels assigned to IOCs based on enrichment.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatLevel {
    Clean,
    Moderate,
    Suspicious,
    Malicious,
    Unknown,
}

impl ThreatLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThreatLevel::Clean => "CLEAN",
            ThreatLevel::Moderate => "MODERATE",
            ThreatLevel::Suspicious => "SUSPICIOUS",
            ThreatLevel::Malicious => "MALICIOUS",
            ThreatLevel::Unknown => "UNKNOWN",
        }
    }
}

/// Summary of enrichment results for one IOC.
#[derive(Debug, Clone)]
pub struct EnrichmentSummary {
    pub threat_level: ThreatLevel,
    pub score: f32,
    pub provider_results: HashMap<String, serde_json::Value>,
}

/// Wraps the intel gRPC client for IOC enrichment.
pub struct IntelGrpcClient {
    inner: IntelEnricherClient<Channel>,
}

impl IntelGrpcClient {
    pub fn new(channel: Channel) -> Self {
        Self {
            inner: IntelEnricherClient::new(channel),
        }
    }

    /// Enrich a single IOC via deepmail-intel. Best-effort: errors → Unknown.
    pub async fn enrich_ioc(
        &self,
        ioc_value: &str,
        ioc_type: &str,
    ) -> EnrichmentSummary {
        // Only enrich enrichable types
        if !matches!(ioc_type, "ip" | "domain" | "url" | "hash") {
            return EnrichmentSummary {
                threat_level: ThreatLevel::Unknown,
                score: 0.0,
                provider_results: HashMap::new(),
            };
        }

        let request = tonic::Request::new(EnrichIocRequest {
            ioc_value: ioc_value.to_string(),
            ioc_type: ioc_type.to_string(),
            providers: vec![], // all configured providers
            force_refresh: false,
        });

        let mut client = self.inner.clone();
        match client.enrich_ioc(request).await {
            Ok(resp) => {
                let r = resp.into_inner();
                let score = r.max_score;
                let is_malicious = r.is_malicious;

                let threat_level = if is_malicious {
                    ThreatLevel::Malicious
                } else if score >= 0.5 {
                    ThreatLevel::Suspicious
                } else if score >= 0.25 {
                    ThreatLevel::Moderate
                } else {
                    ThreatLevel::Clean
                };

                // Parse provider results from JSON strings
                let mut provider_results = HashMap::new();
                for (k, v) in &r.provider_results_json {
                    match serde_json::from_str(v) {
                        Ok(val) => { provider_results.insert(k.clone(), val); }
                        Err(_) => {
                            provider_results.insert(
                                k.clone(),
                                serde_json::Value::String(v.clone()),
                            );
                        }
                    }
                }

                EnrichmentSummary {
                    threat_level,
                    score,
                    provider_results,
                }
            }
            Err(e) => {
                tracing::warn!(
                    ioc_value = ioc_value,
                    ioc_type = ioc_type,
                    error = %e,
                    "intel enrichment failed, marking UNKNOWN"
                );
                EnrichmentSummary {
                    threat_level: ThreatLevel::Unknown,
                    score: 0.0,
                    provider_results: HashMap::new(),
                }
            }
        }
    }
}
