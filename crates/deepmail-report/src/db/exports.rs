use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExportRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub report_format: String,
    pub s3_key: String,
    pub file_size_bytes: i64,
    pub generated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub async fn upsert_export(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    report_format: &str,
    s3_key: &str,
    file_size_bytes: i64,
) -> Result<ExportRow, sqlx::Error> {
    sqlx::query_as!(
        ExportRow,
        r#"INSERT INTO report_exports
             (email_id, tenant_id, report_format, s3_key, file_size_bytes, expires_at)
           VALUES ($1, $2, $3, $4, $5, now() + INTERVAL '30 days')
           ON CONFLICT (email_id, report_format) DO UPDATE SET
             s3_key = EXCLUDED.s3_key,
             file_size_bytes = EXCLUDED.file_size_bytes,
             generated_at = now(),
             expires_at = now() + INTERVAL '30 days'
           RETURNING id, email_id, tenant_id, report_format,
                     s3_key, file_size_bytes, generated_at, expires_at, created_at"#,
        email_id,
        tenant_id,
        report_format,
        s3_key,
        file_size_bytes,
    )
    .fetch_one(pool)
    .await
}

pub async fn get_by_email_format(
    pool: &PgPool,
    email_id: Uuid,
    report_format: &str,
) -> Result<Option<ExportRow>, sqlx::Error> {
    sqlx::query_as!(
        ExportRow,
        r#"SELECT id, email_id, tenant_id, report_format,
                  s3_key, file_size_bytes, generated_at, expires_at, created_at
           FROM report_exports
           WHERE email_id = $1 AND report_format = $2
           LIMIT 1"#,
        email_id,
        report_format,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_by_email(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<ExportRow>, sqlx::Error> {
    sqlx::query_as!(
        ExportRow,
        r#"SELECT id, email_id, tenant_id, report_format,
                  s3_key, file_size_bytes, generated_at, expires_at, created_at
           FROM report_exports
           WHERE email_id = $1
           ORDER BY report_format"#,
        email_id,
    )
    .fetch_all(pool)
    .await
}
