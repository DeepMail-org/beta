use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_log(
    pool: &PgPool,
    tenant_id: Uuid,
    email_id: Uuid,
    event_type: &str,
    channel: &str,
    status: &str,
    recipient: Option<&str>,
    payload: Option<&serde_json::Value>,
    error_message: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO notification_logs
             (tenant_id, email_id, event_type, channel, status, recipient, payload, error_message, attempt_count)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 1)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(email_id)
    .bind(event_type)
    .bind(channel)
    .bind(status)
    .bind(recipient)
    .bind(payload)
    .bind(error_message)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn update_log_status(
    pool: &PgPool,
    log_id: Uuid,
    status: &str,
    attempt_count: i32,
    error_message: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE notification_logs
         SET status = $1, attempt_count = $2, error_message = $3, updated_at = now()
         WHERE id = $4",
    )
    .bind(status)
    .bind(attempt_count)
    .bind(error_message)
    .bind(log_id)
    .execute(pool)
    .await?;

    Ok(())
}
