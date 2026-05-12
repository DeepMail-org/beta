use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct SyncLogRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub nodes_created: i32,
    pub edges_created: i32,
    pub sync_status: String,
    pub error_message: Option<String>,
    pub synced_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<SyncLogRow>, sqlx::Error> {
    sqlx::query_as::<_, SyncLogRow>(
        "SELECT id, email_id, tenant_id, nodes_created, edges_created, \
                sync_status, error_message, synced_at, created_at \
         FROM graph_sync_log WHERE email_id = $1",
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_sync_log(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    nodes_created: i32,
    edges_created: i32,
    status: &str,
    error_message: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO graph_sync_log (email_id, tenant_id, nodes_created, edges_created, sync_status, error_message, synced_at) \
         VALUES ($1, $2, $3, $4, $5, $6, now()) \
         ON CONFLICT (email_id) DO UPDATE SET \
             nodes_created = EXCLUDED.nodes_created, \
             edges_created = EXCLUDED.edges_created, \
             sync_status = EXCLUDED.sync_status, \
             error_message = EXCLUDED.error_message, \
             synced_at = now() \
         RETURNING id",
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(nodes_created)
    .bind(edges_created)
    .bind(status)
    .bind(error_message)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}
