use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct InvoiceRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub billing_period: String,
    pub razorpay_id: Option<String>,
    pub status: String,
    pub total_paise: i64,
    pub line_items_json: serde_json::Value,
    pub issued_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_invoice(
    pool: &PgPool,
    tenant_id: Uuid,
    period: &str,
    razorpay_id: Option<&str>,
    status: &str,
    total_paise: i64,
    line_items_json: &serde_json::Value,
    due_at: Option<DateTime<Utc>>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO invoices (tenant_id, billing_period, razorpay_id, status, total_paise, line_items_json, issued_at, due_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, now(), $7, now())
         ON CONFLICT (tenant_id, billing_period)
         DO UPDATE SET razorpay_id = COALESCE(EXCLUDED.razorpay_id, invoices.razorpay_id),
                       status = EXCLUDED.status,
                       total_paise = EXCLUDED.total_paise,
                       line_items_json = EXCLUDED.line_items_json,
                       issued_at = COALESCE(invoices.issued_at, now()),
                       due_at = COALESCE(EXCLUDED.due_at, invoices.due_at),
                       updated_at = now()
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(period)
    .bind(razorpay_id)
    .bind(status)
    .bind(total_paise)
    .bind(line_items_json)
    .bind(due_at)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn get_by_period(
    pool: &PgPool,
    tenant_id: Uuid,
    period: &str,
) -> Result<Option<InvoiceRow>, sqlx::Error> {
    sqlx::query_as::<_, InvoiceRow>(
        "SELECT id, tenant_id, billing_period, razorpay_id, status, total_paise,
                line_items_json, issued_at, paid_at, due_at, created_at, updated_at
         FROM invoices
         WHERE tenant_id = $1 AND billing_period = $2",
    )
    .bind(tenant_id)
    .bind(period)
    .fetch_optional(pool)
    .await
}

pub async fn update_status(
    pool: &PgPool,
    razorpay_id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE invoices SET status = $1, updated_at = now() WHERE razorpay_id = $2",
    )
    .bind(status)
    .bind(razorpay_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_status_paid(pool: &PgPool, razorpay_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE invoices SET status = 'paid', paid_at = now(), updated_at = now() WHERE razorpay_id = $1",
    )
    .bind(razorpay_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_razorpay_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    payload_json: &serde_json::Value,
) -> Result<bool, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "INSERT INTO razorpay_events (event_id, event_type, payload)
         VALUES ($1, $2, $3)
         ON CONFLICT (event_id) DO NOTHING
         RETURNING id",
    )
    .bind(event_id)
    .bind(event_type)
    .bind(payload_json)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

pub async fn mark_razorpay_processed(pool: &PgPool, event_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE razorpay_events SET processed = true WHERE event_id = $1")
        .bind(event_id)
        .execute(pool)
        .await?;

    Ok(())
}
