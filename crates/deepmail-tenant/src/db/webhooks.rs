//! CRUD for razorpay_events table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::TenantError;

/// Insert a Razorpay webhook event.
/// Returns Ok(true) if inserted, Ok(false) if duplicate (already exists).
pub async fn insert_webhook_event(
    pool: &PgPool,
    event_id: &str,
    event_type: &str,
    tenant_id: Option<Uuid>,
    payload: &serde_json::Value,
) -> Result<bool, TenantError> {
    let result = sqlx::query!(
        r#"
        INSERT INTO razorpay_events
          (event_id, event_type, tenant_id, payload)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (event_id) DO NOTHING
        "#,
        event_id,
        event_type,
        tenant_id,
        payload,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;

    Ok(result.rows_affected() > 0)
}

/// Mark a webhook event as processed.
pub async fn mark_event_processed(
    pool: &PgPool,
    event_id: &str,
    error: Option<&str>,
) -> Result<(), TenantError> {
    sqlx::query!(
        r#"
        UPDATE razorpay_events
        SET processed    = true,
            processed_at = now(),
            error        = $2
        WHERE event_id = $1
        "#,
        event_id,
        error,
    )
    .execute(pool)
    .await
    .map_err(TenantError::Database)?;
    Ok(())
}
