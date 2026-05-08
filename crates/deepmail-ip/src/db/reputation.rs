/// ip_reputation table operations.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IpError;

#[derive(Debug, Clone)]
pub struct IpReputationRow {
    pub id: Uuid,
    pub ip_address: String,
    pub threat_score: f32,
    pub threat_verdict: String,
    pub feed_hits: Vec<String>,
    pub abuse_score: Option<i32>,
    pub shodan_ports: Vec<i32>,
    pub shodan_tags: Vec<String>,
    pub shodan_vulns: Vec<String>,
    pub shodan_org: Option<String>,
    pub pdns_hostnames: Vec<String>,
    pub pdns_first_seen: Option<DateTime<Utc>>,
    pub pdns_last_seen: Option<DateTime<Utc>>,
    pub bgp_prefix: Option<String>,
    pub bgp_asn: Option<i32>,
    pub bgp_holder: Option<String>,
    pub bgp_announced: bool,
    pub is_bogon: bool,
    pub sighting_count: i32,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub enriched_at: Option<DateTime<Utc>>,
}

/// Upsert an IP reputation record. On conflict, merges all enrichment fields
/// and increments sighting_count.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_reputation(
    pool: &PgPool,
    ip_address: &str,
    threat_score: f32,
    threat_verdict: &str,
    feed_hits: &[String],
    abuse_score: Option<i32>,
    shodan_ports: &[i32],
    shodan_tags: &[String],
    shodan_vulns: &[String],
    shodan_org: Option<&str>,
    pdns_hostnames: &[String],
    pdns_first_seen: Option<DateTime<Utc>>,
    pdns_last_seen: Option<DateTime<Utc>>,
    bgp_prefix: Option<&str>,
    bgp_asn: Option<i32>,
    bgp_holder: Option<&str>,
    bgp_announced: bool,
    is_bogon: bool,
) -> Result<Uuid, IpError> {
    let row = sqlx::query(
        r#"INSERT INTO ip_reputation (
               ip_address, threat_score, threat_verdict, feed_hits,
               abuse_score, shodan_ports, shodan_tags, shodan_vulns, shodan_org,
               pdns_hostnames, pdns_first_seen, pdns_last_seen,
               bgp_prefix, bgp_asn, bgp_holder, bgp_announced,
               is_bogon, enriched_at
           )
           VALUES (
               $1::inet, $2, $3, $4,
               $5, $6, $7, $8, $9,
               $10, $11, $12,
               $13::cidr, $14, $15, $16,
               $17, now()
           )
           ON CONFLICT (ip_address)
           DO UPDATE SET
               threat_score    = EXCLUDED.threat_score,
               threat_verdict  = EXCLUDED.threat_verdict,
               feed_hits       = EXCLUDED.feed_hits,
               abuse_score     = EXCLUDED.abuse_score,
               shodan_ports    = EXCLUDED.shodan_ports,
               shodan_tags     = EXCLUDED.shodan_tags,
               shodan_vulns    = EXCLUDED.shodan_vulns,
               shodan_org      = EXCLUDED.shodan_org,
               pdns_hostnames  = EXCLUDED.pdns_hostnames,
               pdns_first_seen = COALESCE(ip_reputation.pdns_first_seen, EXCLUDED.pdns_first_seen),
               pdns_last_seen  = EXCLUDED.pdns_last_seen,
               bgp_prefix      = EXCLUDED.bgp_prefix,
               bgp_asn         = EXCLUDED.bgp_asn,
               bgp_holder      = EXCLUDED.bgp_holder,
               bgp_announced   = EXCLUDED.bgp_announced,
               is_bogon        = EXCLUDED.is_bogon,
               sighting_count  = ip_reputation.sighting_count + 1,
               last_seen       = now(),
               enriched_at     = now(),
               updated_at      = now()
           RETURNING id"#,
    )
    .bind(ip_address)
    .bind(threat_score)
    .bind(threat_verdict)
    .bind(feed_hits)
    .bind(abuse_score)
    .bind(shodan_ports)
    .bind(shodan_tags)
    .bind(shodan_vulns)
    .bind(shodan_org)
    .bind(pdns_hostnames)
    .bind(pdns_first_seen)
    .bind(pdns_last_seen)
    .bind(bgp_prefix)
    .bind(bgp_asn)
    .bind(bgp_holder)
    .bind(bgp_announced)
    .bind(is_bogon)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok(row.get("id"))
}

/// Get a single IP reputation by IP address.
pub async fn get_by_ip(pool: &PgPool, ip_address: &str) -> Result<Option<IpReputationRow>, IpError> {
    let row = sqlx::query(
        r#"SELECT id, CAST(ip_address AS TEXT) AS ip_address,
                  threat_score, threat_verdict, feed_hits, abuse_score,
                  shodan_ports, shodan_tags, shodan_vulns, shodan_org,
                  pdns_hostnames, pdns_first_seen, pdns_last_seen,
                  CAST(bgp_prefix AS TEXT) AS bgp_prefix, bgp_asn, bgp_holder, bgp_announced,
                  is_bogon, sighting_count, first_seen, last_seen, enriched_at
           FROM ip_reputation
           WHERE ip_address = $1::inet"#,
    )
    .bind(ip_address)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            use sqlx::Row;
            Ok(Some(IpReputationRow {
                id: r.get("id"),
                ip_address: r.get("ip_address"),
                threat_score: r.get("threat_score"),
                threat_verdict: r.get("threat_verdict"),
                feed_hits: r.get("feed_hits"),
                abuse_score: r.get("abuse_score"),
                shodan_ports: r.get("shodan_ports"),
                shodan_tags: r.get("shodan_tags"),
                shodan_vulns: r.get("shodan_vulns"),
                shodan_org: r.get("shodan_org"),
                pdns_hostnames: r.get("pdns_hostnames"),
                pdns_first_seen: r.get("pdns_first_seen"),
                pdns_last_seen: r.get("pdns_last_seen"),
                bgp_prefix: r.get("bgp_prefix"),
                bgp_asn: r.get("bgp_asn"),
                bgp_holder: r.get("bgp_holder"),
                bgp_announced: r.get("bgp_announced"),
                is_bogon: r.get("is_bogon"),
                sighting_count: r.get("sighting_count"),
                first_seen: r.get("first_seen"),
                last_seen: r.get("last_seen"),
                enriched_at: r.get("enriched_at"),
            }))
        }
        None => Ok(None),
    }
}

/// Bulk get up to 500 IP reputations.
pub async fn bulk_get(pool: &PgPool, ips: &[String]) -> Result<Vec<IpReputationRow>, IpError> {
    if ips.is_empty() {
        return Ok(Vec::new());
    }

    // Build a dynamic query with ANY($1)
    // We pass IPs as TEXT[] and cast in the query
    let rows = sqlx::query(
        r#"SELECT id, CAST(ip_address AS TEXT) AS ip_address,
                  threat_score, threat_verdict, feed_hits, abuse_score,
                  shodan_ports, shodan_tags, shodan_vulns, shodan_org,
                  pdns_hostnames, pdns_first_seen, pdns_last_seen,
                  CAST(bgp_prefix AS TEXT) AS bgp_prefix, bgp_asn, bgp_holder, bgp_announced,
                  is_bogon, sighting_count, first_seen, last_seen, enriched_at
           FROM ip_reputation
           WHERE ip_address = ANY($1::inet[])"#,
    )
    .bind(ips)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let results = rows
        .iter()
        .map(|r| IpReputationRow {
            id: r.get("id"),
            ip_address: r.get("ip_address"),
            threat_score: r.get("threat_score"),
            threat_verdict: r.get("threat_verdict"),
            feed_hits: r.get("feed_hits"),
            abuse_score: r.get("abuse_score"),
            shodan_ports: r.get("shodan_ports"),
            shodan_tags: r.get("shodan_tags"),
            shodan_vulns: r.get("shodan_vulns"),
            shodan_org: r.get("shodan_org"),
            pdns_hostnames: r.get("pdns_hostnames"),
            pdns_first_seen: r.get("pdns_first_seen"),
            pdns_last_seen: r.get("pdns_last_seen"),
            bgp_prefix: r.get("bgp_prefix"),
            bgp_asn: r.get("bgp_asn"),
            bgp_holder: r.get("bgp_holder"),
            bgp_announced: r.get("bgp_announced"),
            is_bogon: r.get("is_bogon"),
            sighting_count: r.get("sighting_count"),
            first_seen: r.get("first_seen"),
            last_seen: r.get("last_seen"),
            enriched_at: r.get("enriched_at"),
        })
        .collect();

    Ok(results)
}
