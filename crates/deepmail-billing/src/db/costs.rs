use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct EventCostRow {
    pub id: Uuid,
    pub event_type: String,
    pub cost_paise: i32,
    pub description: String,
    pub is_active: bool,
}

pub async fn get_cost(pool: &PgPool, event_type: &str) -> Result<Option<i32>, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as(
        "SELECT cost_paise FROM billing_event_costs WHERE event_type = $1 AND is_active = true",
    )
    .bind(event_type)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|(cost,)| cost))
}

pub async fn list_all_costs(pool: &PgPool) -> Result<Vec<EventCostRow>, sqlx::Error> {
    sqlx::query_as::<_, EventCostRow>(
        "SELECT id, event_type, cost_paise, description, is_active FROM billing_event_costs ORDER BY event_type",
    )
    .fetch_all(pool)
    .await
}
