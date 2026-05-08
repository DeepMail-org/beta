/// ip_pdns_records table queries (upserts happen in pdns.rs).

use sqlx::PgPool;

use crate::error::IpError;

/// Get all passive DNS hostnames for an IP from the database.
pub async fn get_pdns_for_ip(pool: &PgPool, ip: &str) -> Result<Vec<String>, IpError> {
    let rows = sqlx::query(
        r#"SELECT hostname FROM ip_pdns_records
           WHERE ip_address = $1::inet
           ORDER BY last_seen DESC NULLS LAST"#,
    )
    .bind(ip)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let hostnames: Vec<String> = rows.iter().map(|r| r.get("hostname")).collect();
    Ok(hostnames)
}
