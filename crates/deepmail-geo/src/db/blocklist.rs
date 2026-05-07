use sqlx::PgPool;
use uuid::Uuid;

use crate::error::GeoError;

pub struct FeedRow {
    pub id: Uuid,
    pub feed_name: String,
    pub feed_url: String,
    pub is_active: bool,
}

pub async fn list_active_feeds(pool: &PgPool) -> Result<Vec<FeedRow>, GeoError> {
    let rows = sqlx::query_as!(
        FeedRow,
        r#"SELECT id, feed_name, feed_url, is_active
           FROM geo_blocklist_feeds
           WHERE is_active = true"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
