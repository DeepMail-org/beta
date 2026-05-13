use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct UsageLine {
    pub event_type: String,
    pub count: i64,
    pub total_paise: i64,
}

pub async fn insert_meter_event(
    pool: &PgPool,
    idempotency_key: Uuid,
    tenant_id: Uuid,
    email_id: Uuid,
    event_type: &str,
    cost_paise: i32,
    billing_period: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "INSERT INTO meter_events (idempotency_key, tenant_id, email_id, event_type, cost_paise, billing_period)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (idempotency_key) DO NOTHING
         RETURNING id",
    )
    .bind(idempotency_key)
    .bind(tenant_id)
    .bind(email_id)
    .bind(event_type)
    .bind(cost_paise)
    .bind(billing_period)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(id,)| id))
}

pub async fn get_usage_summary(
    pool: &PgPool,
    tenant_id: Uuid,
    period: &str,
) -> Result<Vec<UsageLine>, sqlx::Error> {
    sqlx::query_as::<_, UsageLine>(
        "SELECT event_type, COUNT(*) AS count, COALESCE(SUM(cost_paise), 0) AS total_paise
         FROM meter_events
         WHERE tenant_id = $1 AND billing_period = $2
         GROUP BY event_type
         ORDER BY event_type",
    )
    .bind(tenant_id)
    .bind(period)
    .fetch_all(pool)
    .await
}
