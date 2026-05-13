use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DigestRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub frequency: String,
    pub recipient_emails: Vec<String>,
    pub last_sent_at: Option<DateTime<Utc>>,
    pub next_send_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_schedule(
    pool: &PgPool,
    tenant_id: Uuid,
    frequency: &str,
    recipient_emails: &[String],
    next_send_at: Option<DateTime<Utc>>,
) -> Result<DigestRow, sqlx::Error> {
    sqlx::query_as!(
        DigestRow,
        r#"INSERT INTO digest_schedules
             (tenant_id, frequency, recipient_emails, next_send_at, is_active)
           VALUES ($1, $2, $3, $4, true)
           ON CONFLICT (tenant_id) DO UPDATE SET
             frequency = EXCLUDED.frequency,
             recipient_emails = EXCLUDED.recipient_emails,
             next_send_at = EXCLUDED.next_send_at,
             is_active = CASE WHEN EXCLUDED.frequency = 'never' THEN false ELSE true END,
             updated_at = now()
           RETURNING id, tenant_id, frequency, recipient_emails,
                     last_sent_at, next_send_at, is_active, created_at, updated_at"#,
        tenant_id,
        frequency,
        recipient_emails,
        next_send_at,
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Option<DigestRow>, sqlx::Error> {
    sqlx::query_as!(
        DigestRow,
        r#"SELECT id, tenant_id, frequency, recipient_emails,
                  last_sent_at, next_send_at, is_active, created_at, updated_at
           FROM digest_schedules
           WHERE tenant_id = $1
           LIMIT 1"#,
        tenant_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_due(
    pool: &PgPool,
    now: DateTime<Utc>,
) -> Result<Vec<DigestRow>, sqlx::Error> {
    sqlx::query_as!(
        DigestRow,
        r#"SELECT id, tenant_id, frequency, recipient_emails,
                  last_sent_at, next_send_at, is_active, created_at, updated_at
           FROM digest_schedules
           WHERE is_active = true AND next_send_at <= $1
           ORDER BY next_send_at"#,
        now,
    )
    .fetch_all(pool)
    .await
}
