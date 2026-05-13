use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

pub async fn insert_session(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Uuid,
    session_token: &str,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO websocket_sessions (tenant_id, user_id, session_token)
         VALUES ($1, $2, $3)
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(session_token)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

pub async fn mark_disconnected(pool: &PgPool, session_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE websocket_sessions
         SET is_active = false, disconnected_at = now()
         WHERE id = $1",
    )
    .bind(session_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_active_by_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Vec<SessionRow>, sqlx::Error> {
    sqlx::query_as::<_, SessionRow>(
        "SELECT id, tenant_id, user_id, session_token, connected_at, disconnected_at, is_active
         FROM websocket_sessions
         WHERE tenant_id = $1 AND is_active = true",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
}
