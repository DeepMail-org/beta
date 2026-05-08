/// Passive DNS correlation via CIRCL pDNS API.

use std::net::IpAddr;

use chrono::{DateTime, TimeZone, Utc};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::IpError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdnsRecord {
    pub rrname: String,
    pub rrtype: String,
    pub rdata: String,
    #[serde(default)]
    pub time_first: Option<i64>,
    #[serde(default)]
    pub time_last: Option<i64>,
}

fn redis_key(ip: IpAddr) -> String {
    format!("deepmail:ip:pdns:{ip}")
}

/// Fetch passive DNS records from CIRCL and upsert into database.
pub async fn fetch_pdns(
    client: &reqwest::Client,
    pool: &PgPool,
    redis: &mut ConnectionManager,
    ip: IpAddr,
) -> Result<Vec<String>, IpError> {
    // Check Redis cache first
    let key = redis_key(ip);
    let cached: Option<String> = redis.get(&key).await?;
    if let Some(json) = cached {
        if let Ok(hostnames) = serde_json::from_str::<Vec<String>>(&json) {
            return Ok(hostnames);
        }
    }

    let url = format!("https://www.circl.lu/pdns/query/{ip}");
    let resp = client.get(&url).send().await;

    let body = match resp {
        Ok(r) => {
            if !r.status().is_success() {
                tracing::debug!(ip = %ip, status = %r.status(), "CIRCL pDNS non-success");
                return Ok(Vec::new());
            }
            r.text().await.unwrap_or_default()
        }
        Err(e) => {
            tracing::debug!(ip = %ip, error = %e, "CIRCL pDNS request failed");
            return Ok(Vec::new());
        }
    };

    // CIRCL returns newline-delimited JSON, NOT a JSON array
    let mut hostnames = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let record: PdnsRecord = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                tracing::debug!(error = %e, "skipping unparseable pDNS line");
                continue;
            }
        };

        // Only A and AAAA records
        if record.rrtype != "A" && record.rrtype != "AAAA" {
            continue;
        }

        let first_seen: Option<DateTime<Utc>> = record.time_first.and_then(|t| Utc.timestamp_opt(t, 0).single());
        let last_seen: Option<DateTime<Utc>> = record.time_last.and_then(|t| Utc.timestamp_opt(t, 0).single());

        // Upsert into database
        upsert_pdns_record(pool, ip, &record.rrname, &record.rrtype, first_seen, last_seen).await?;

        if !hostnames.contains(&record.rrname) {
            hostnames.push(record.rrname);
        }
    }

    // Cache hostnames in Redis with 1h TTL
    if let Ok(json) = serde_json::to_string(&hostnames) {
        let _: Result<(), _> = redis.set_ex(&key, &json, 3600).await;
    }

    Ok(hostnames)
}

async fn upsert_pdns_record(
    pool: &PgPool,
    ip: IpAddr,
    hostname: &str,
    record_type: &str,
    first_seen: Option<DateTime<Utc>>,
    last_seen: Option<DateTime<Utc>>,
) -> Result<(), IpError> {
    sqlx::query(
        r#"INSERT INTO ip_pdns_records (ip_address, hostname, record_type, first_seen, last_seen)
           VALUES ($1::inet, $2, $3, $4, $5)
           ON CONFLICT (ip_address, hostname)
           DO UPDATE SET last_seen = COALESCE(EXCLUDED.last_seen, ip_pdns_records.last_seen),
                         record_type = EXCLUDED.record_type"#,
    )
    .bind(ip.to_string())
    .bind(hostname)
    .bind(record_type)
    .bind(first_seen)
    .bind(last_seen)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get cached pDNS hostnames from Redis.
pub async fn get_pdns_cached(
    redis: &mut ConnectionManager,
    ip: IpAddr,
) -> Result<Option<Vec<String>>, IpError> {
    let key = redis_key(ip);
    let cached: Option<String> = redis.get(&key).await?;
    match cached {
        Some(json) => {
            let hostnames: Vec<String> =
                serde_json::from_str(&json).map_err(|e| IpError::Parse(e.to_string()))?;
            Ok(Some(hostnames))
        }
        None => Ok(None),
    }
}
