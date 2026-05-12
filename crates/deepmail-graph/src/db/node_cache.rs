use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub struct NodeCacheRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub node_type: String,
    pub node_value: String,
    pub neo4j_id: String,
    pub properties_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn upsert_node_cache(
    pool: &PgPool,
    tenant_id: Uuid,
    node_type: &str,
    node_value: &str,
    neo4j_id: &str,
    properties_json: &serde_json::Value,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO graph_node_cache (tenant_id, node_type, node_value, neo4j_id, properties_json) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (tenant_id, node_type, node_value) DO UPDATE SET \
             neo4j_id = EXCLUDED.neo4j_id, \
             properties_json = EXCLUDED.properties_json, \
             updated_at = now() \
         RETURNING id",
    )
    .bind(tenant_id)
    .bind(node_type)
    .bind(node_value)
    .bind(neo4j_id)
    .bind(properties_json)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

pub async fn get_node(
    pool: &PgPool,
    tenant_id: Uuid,
    node_type: &str,
    node_value: &str,
) -> Result<Option<NodeCacheRow>, sqlx::Error> {
    sqlx::query_as::<_, NodeCacheRow>(
        "SELECT id, tenant_id, node_type, node_value, neo4j_id, properties_json, created_at, updated_at \
         FROM graph_node_cache WHERE tenant_id = $1 AND node_type = $2 AND node_value = $3",
    )
    .bind(tenant_id)
    .bind(node_type)
    .bind(node_value)
    .fetch_optional(pool)
    .await
}
