/// Campaign database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IocError;

/// Campaign row from campaign_clusters.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CampaignRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub campaign_name: String,
    pub status: String,
    pub ioc_fingerprint: Vec<String>,
    pub member_count: i32,
    pub first_email_at: chrono::DateTime<chrono::Utc>,
    pub last_email_at: chrono::DateTime<chrono::Utc>,
}

/// Get campaign by id.
pub async fn get_by_id(pool: &PgPool, campaign_id: Uuid) -> Result<Option<CampaignRow>, IocError> {
    let row = sqlx::query_as::<_, CampaignRow>(
        r#"SELECT id, tenant_id, campaign_name, status, ioc_fingerprint,
                  member_count, first_email_at, last_email_at
           FROM campaign_clusters WHERE id = $1"#,
    )
    .bind(campaign_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Get campaign status for an email's campaign.
pub async fn get_campaign_for_email(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<(Uuid, String)>, IocError> {
    let row: Option<(Uuid, String)> = sqlx::query_as(
        r#"SELECT c.id, c.status
           FROM campaign_clusters c
           JOIN campaign_members m ON m.campaign_id = c.id
           WHERE m.email_id = $1
           LIMIT 1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
