/// ip_blocklist_entries table operations.

use sqlx::PgPool;

use crate::error::IpError;

/// Check an IP against all blocklist entries (both exact INET match and CIDR containment).
/// Returns list of feed names where this IP matches.
pub async fn check_ip(pool: &PgPool, ip: &str) -> Result<Vec<String>, IpError> {
    let rows = sqlx::query(
        r#"SELECT f.feed_name
           FROM ip_blocklist_entries e
           JOIN ip_blocklist_feeds f ON f.id = e.feed_id
           WHERE e.ip_address = $1::inet
              OR e.cidr_range >>= $1::inet"#,
    )
    .bind(ip)
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let feeds: Vec<String> = rows.iter().map(|r| r.get("feed_name")).collect();
    Ok(feeds)
}

/// Get all feed names for a given IP from the database.
/// This is useful for building the SignalSet for scoring.
pub async fn get_feeds_for_ip(pool: &PgPool, ip: &str) -> Result<Vec<String>, IpError> {
    check_ip(pool, ip).await
}
