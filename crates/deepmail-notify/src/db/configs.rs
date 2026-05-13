use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct NotifyConfigRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub webhook_active: bool,
    pub smtp_enabled: bool,
    pub min_severity: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn get_config(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Option<NotifyConfigRow>, sqlx::Error> {
    sqlx::query_as::<_, NotifyConfigRow>(
        "SELECT id, tenant_id, webhook_url, webhook_secret, webhook_active,
                smtp_enabled, min_severity, created_at, updated_at
         FROM notification_configs
         WHERE tenant_id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_config(
    pool: &PgPool,
    tenant_id: Uuid,
    webhook_url: &str,
    webhook_secret: &str,
    smtp_enabled: bool,
    min_severity: &str,
) -> Result<Uuid, sqlx::Error> {
    let webhook_active = !webhook_url.is_empty();
    let wh_url: Option<&str> = if webhook_url.is_empty() {
        None
    } else {
        Some(webhook_url)
    };
    let wh_secret: Option<&str> = if webhook_secret.is_empty() {
        None
    } else {
        Some(webhook_secret)
    };

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO notification_configs (tenant_id, webhook_url, webhook_secret, webhook_active, smtp_enabled, min_severity)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (tenant_id) DO UPDATE SET
             webhook_url    = EXCLUDED.webhook_url,
             webhook_secret = EXCLUDED.webhook_secret,
             webhook_active = EXCLUDED.webhook_active,
             smtp_enabled   = EXCLUDED.smtp_enabled,
             min_severity   = EXCLUDED.min_severity,
             updated_at     = now()
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(wh_url)
    .bind(wh_secret)
    .bind(webhook_active)
    .bind(smtp_enabled)
    .bind(min_severity)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
