/// ip_shodan_cache table operations.

use chrono::Utc;
use sqlx::PgPool;

use crate::error::IpError;
use crate::shodan::ShodanResponse;

/// Upsert Shodan cache into the database.
pub async fn upsert_shodan_cache(
    pool: &PgPool,
    ip_address: &str,
    resp: &ShodanResponse,
) -> Result<(), IpError> {
    let ports = resp.ports.clone().unwrap_or_default();
    let tags = resp.tags.clone().unwrap_or_default();
    let vulns: Vec<String> = resp
        .vulns
        .as_ref()
        .map(|v| v.keys().cloned().collect())
        .unwrap_or_default();
    let hostnames = resp.hostnames.clone().unwrap_or_default();
    let expires_at = Utc::now() + chrono::Duration::hours(24);

    sqlx::query(
        r#"INSERT INTO ip_shodan_cache (
               ip_address, ports, tags, vulns, org, isp, os,
               hostnames, country_code, asn, last_update, expires_at
           )
           VALUES ($1::inet, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
           ON CONFLICT (ip_address)
           DO UPDATE SET
               ports        = EXCLUDED.ports,
               tags         = EXCLUDED.tags,
               vulns        = EXCLUDED.vulns,
               org          = EXCLUDED.org,
               isp          = EXCLUDED.isp,
               os           = EXCLUDED.os,
               hostnames    = EXCLUDED.hostnames,
               country_code = EXCLUDED.country_code,
               asn          = EXCLUDED.asn,
               last_update  = EXCLUDED.last_update,
               fetched_at   = now(),
               expires_at   = EXCLUDED.expires_at"#,
    )
    .bind(ip_address)
    .bind(&ports)
    .bind(&tags)
    .bind(&vulns)
    .bind(resp.org.as_deref())
    .bind(resp.isp.as_deref())
    .bind(resp.os.as_deref())
    .bind(&hostnames)
    .bind(resp.country_code.as_deref())
    .bind(resp.asn.as_deref())
    .bind(resp.last_update.as_deref())
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get cached Shodan data from DB (only if not expired).
pub async fn get_shodan_cache(
    pool: &PgPool,
    ip_address: &str,
) -> Result<Option<ShodanResponse>, IpError> {
    let row = sqlx::query(
        r#"SELECT ports, tags, vulns, org, isp, os, hostnames,
                  country_code, asn, last_update
           FROM ip_shodan_cache
           WHERE ip_address = $1::inet AND expires_at > now()"#,
    )
    .bind(ip_address)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            use sqlx::Row;
            let vulns_list: Vec<String> = r.get("vulns");
            let vulns_map = vulns_list
                .into_iter()
                .map(|cve| (cve, serde_json::Value::Null))
                .collect();

            Ok(Some(ShodanResponse {
                ip_str: Some(ip_address.to_string()),
                ports: Some(r.get("ports")),
                tags: Some(r.get("tags")),
                vulns: Some(vulns_map),
                org: r.get("org"),
                isp: r.get("isp"),
                os: r.get("os"),
                last_update: r.get("last_update"),
                hostnames: Some(r.get("hostnames")),
                country_code: r.get("country_code"),
                asn: r.get("asn"),
            }))
        }
        None => Ok(None),
    }
}
