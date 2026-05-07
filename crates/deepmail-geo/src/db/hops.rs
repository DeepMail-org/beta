use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::GeoError;

pub struct HopRow {
    pub id: Uuid,
    pub analysis_id: Uuid,
    pub hop_index: i32,
    pub ip_address: String,
    pub ip_type: String,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub postal_code: Option<String>,
    pub timezone: Option<String>,
    pub asn: Option<i32>,
    pub as_org: Option<String>,
    pub network_cidr: Option<String>,
    pub is_tor: bool,
    pub is_vpn: bool,
    pub is_botnet: bool,
    pub abuse_score: Option<i32>,
    pub greynoise_class: Option<String>,
    pub ipinfo_raw: Option<serde_json::Value>,
    pub abuseipdb_raw: Option<serde_json::Value>,
    pub greynoise_raw: Option<serde_json::Value>,
    pub enriched_at: Option<DateTime<Utc>>,
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_hop(
    pool: &PgPool,
    analysis_id: Uuid,
    hop_index: i32,
    ip_address: &str,
    ip_type: &str,
    country_code: Option<&str>,
    country_name: Option<&str>,
    city: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    postal_code: Option<&str>,
    timezone: Option<&str>,
    asn: Option<i32>,
    as_org: Option<&str>,
    network_cidr: Option<&str>,
    is_tor: bool,
    is_vpn: bool,
    is_botnet: bool,
    abuse_score: Option<i32>,
    greynoise_class: Option<&str>,
    ipinfo_raw: Option<&serde_json::Value>,
    abuseipdb_raw: Option<&serde_json::Value>,
    greynoise_raw: Option<&serde_json::Value>,
    enriched_at: Option<DateTime<Utc>>,
) -> Result<Uuid, GeoError> {
    let row = sqlx::query(
        r#"INSERT INTO hop_geo_details (
             analysis_id, hop_index, ip_address, ip_type,
             country_code, country_name, city,
             latitude, longitude, postal_code, timezone,
             asn, as_org, network_cidr,
             is_tor, is_vpn, is_botnet,
             abuse_score, greynoise_class,
             ipinfo_raw, abuseipdb_raw, greynoise_raw,
             enriched_at
           ) VALUES (
             $1, $2, $3::inet, $4,
             $5, $6, $7,
             $8, $9, $10, $11,
             $12, $13, $14,
             $15, $16, $17,
             $18, $19,
             $20, $21, $22,
             $23
           )
           RETURNING id"#,
    )
    .bind(analysis_id)
    .bind(hop_index)
    .bind(ip_address)
    .bind(ip_type)
    .bind(country_code)
    .bind(country_name)
    .bind(city)
    .bind(latitude)
    .bind(longitude)
    .bind(postal_code)
    .bind(timezone)
    .bind(asn)
    .bind(as_org)
    .bind(network_cidr)
    .bind(is_tor)
    .bind(is_vpn)
    .bind(is_botnet)
    .bind(abuse_score)
    .bind(greynoise_class)
    .bind(ipinfo_raw)
    .bind(abuseipdb_raw)
    .bind(greynoise_raw)
    .bind(enriched_at)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    let id: Uuid = row.get("id");
    Ok(id)
}

pub async fn list_by_analysis(
    pool: &PgPool,
    analysis_id: Uuid,
) -> Result<Vec<HopRow>, GeoError> {
    let rows = sqlx::query(
        r#"SELECT
             id, analysis_id, hop_index,
             ip_address::text as ip_address,
             ip_type, country_code, country_name, city,
             latitude, longitude, postal_code, timezone,
             asn, as_org, network_cidr,
             is_tor, is_vpn, is_botnet,
             abuse_score, greynoise_class,
             ipinfo_raw, abuseipdb_raw, greynoise_raw,
             enriched_at
           FROM hop_geo_details
           WHERE analysis_id = $1
           ORDER BY hop_index"#,
    )
    .bind(analysis_id)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    Ok(rows
        .iter()
        .map(|r| HopRow {
            id: r.get("id"),
            analysis_id: r.get("analysis_id"),
            hop_index: r.get("hop_index"),
            ip_address: r.get("ip_address"),
            ip_type: r.get("ip_type"),
            country_code: r.get("country_code"),
            country_name: r.get("country_name"),
            city: r.get("city"),
            latitude: r.get("latitude"),
            longitude: r.get("longitude"),
            postal_code: r.get("postal_code"),
            timezone: r.get("timezone"),
            asn: r.get("asn"),
            as_org: r.get("as_org"),
            network_cidr: r.get("network_cidr"),
            is_tor: r.get("is_tor"),
            is_vpn: r.get("is_vpn"),
            is_botnet: r.get("is_botnet"),
            abuse_score: r.get("abuse_score"),
            greynoise_class: r.get("greynoise_class"),
            ipinfo_raw: r.get("ipinfo_raw"),
            abuseipdb_raw: r.get("abuseipdb_raw"),
            greynoise_raw: r.get("greynoise_raw"),
            enriched_at: r.get("enriched_at"),
        })
        .collect())
}
