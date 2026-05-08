pub mod abuseipdb;
pub mod greynoise;
pub mod ipinfo;
pub mod otx;
pub mod shodan;
pub mod virustotal;

/// Common result wrapper returned by all provider lookups.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderResult {
    pub provider: String,
    pub raw_json: serde_json::Value,
    pub score: f32,
}
