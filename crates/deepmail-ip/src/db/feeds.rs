/// ip_blocklist_feeds table operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IpError;

#[derive(Debug, Clone)]
pub struct FeedRow {
    pub id: Uuid,
    pub feed_name: String,
    pub feed_url: String,
    pub threat_category: String,
    pub is_active: bool,
}

/// List all active feeds.
pub async fn list_active_feeds(pool: &PgPool) -> Result<Vec<FeedRow>, IpError> {
    let rows = sqlx::query(
        r#"SELECT id, feed_name, feed_url, threat_category, is_active
           FROM ip_blocklist_feeds
           WHERE is_active = true
           ORDER BY feed_name"#,
    )
    .fetch_all(pool)
    .await?;

    use sqlx::Row;
    let feeds = rows
        .iter()
        .map(|r| FeedRow {
            id: r.get("id"),
            feed_name: r.get("feed_name"),
            feed_url: r.get("feed_url"),
            threat_category: r.get("threat_category"),
            is_active: r.get("is_active"),
        })
        .collect();

    Ok(feeds)
}

/// Mark a feed as successfully fetched with updated entry count.
pub async fn mark_fetched(pool: &PgPool, feed_id: Uuid, entry_count: i32) -> Result<(), IpError> {
    sqlx::query(
        r#"UPDATE ip_blocklist_feeds
           SET last_fetched_at = now(), entry_count = $1
           WHERE id = $2"#,
    )
    .bind(entry_count)
    .bind(feed_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Increment the fetch_errors counter for a feed.
pub async fn increment_error(pool: &PgPool, feed_id: Uuid) -> Result<(), IpError> {
    sqlx::query(
        r#"UPDATE ip_blocklist_feeds
           SET fetch_errors = fetch_errors + 1
           WHERE id = $1"#,
    )
    .bind(feed_id)
    .execute(pool)
    .await?;
    Ok(())
}
