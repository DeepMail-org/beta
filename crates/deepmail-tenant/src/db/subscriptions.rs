//! CRUD for subscriptions table.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::TenantError;

pub struct SubscriptionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub razorpay_subscription_id: Option<String>,
    pub status: String,
    pub current_period_start: Option<DateTime<Utc>>,
    pub current_period_end: Option<DateTime<Utc>>,
}

pub async fn upsert_subscription(
    pool: &PgPool,
    tenant_id: Uuid,
    razorpay_subscription_id: &str,
    razorpay_plan_id: &str,
    status: &str,
    period_start: Option<DateTime<Utc>>,
    period_end: Option<DateTime<Utc>>,
) -> Result<Uuid, TenantError> {
    let row = sqlx::query!(
        r#"
        INSERT INTO subscriptions
          (tenant_id, razorpay_subscription_id, razorpay_plan_id,
           status, current_period_start, current_period_end)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (razorpay_subscription_id) DO UPDATE
          SET status               = EXCLUDED.status,
              razorpay_plan_id     = EXCLUDED.razorpay_plan_id,
              current_period_start = EXCLUDED.current_period_start,
              current_period_end   = EXCLUDED.current_period_end,
              updated_at           = now()
        RETURNING id
        "#,
        tenant_id,
        razorpay_subscription_id,
        razorpay_plan_id,
        status,
        period_start,
        period_end,
    )
    .fetch_one(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(row.id)
}
