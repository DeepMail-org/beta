/// IOC node database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IocError;

/// Row returned from ioc_nodes queries.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IocNodeRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub ioc_type: String,
    pub ioc_value: String,
    pub threat_level: String,
    pub intel_score: f32,
    pub intel_json: serde_json::Value,
    pub sighting_count: i32,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Upsert an IOC node. On conflict: increment sighting, update last_seen, append email_id.
/// Returns the node id.
pub async fn upsert_node(
    pool: &PgPool,
    tenant_id: Uuid,
    ioc_type: &str,
    ioc_value: &str,
    email_id: Uuid,
) -> Result<Uuid, IocError> {
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO ioc_nodes (tenant_id, ioc_type, ioc_value, email_ids)
           VALUES ($1, $2, $3, ARRAY[$4::uuid])
           ON CONFLICT (tenant_id, ioc_type, ioc_value) DO UPDATE
             SET sighting_count = ioc_nodes.sighting_count + 1,
                 last_seen = now(),
                 email_ids = array_append(ioc_nodes.email_ids, $4::uuid),
                 updated_at = now()
           RETURNING id"#,
    )
    .bind(tenant_id)
    .bind(ioc_type)
    .bind(ioc_value)
    .bind(email_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

/// Update enrichment data on a node.
pub async fn update_enrichment(
    pool: &PgPool,
    node_id: Uuid,
    threat_level: &str,
    score: f32,
    intel_json: &serde_json::Value,
) -> Result<(), IocError> {
    sqlx::query(
        r#"UPDATE ioc_nodes
           SET threat_level = $1, intel_score = $2, intel_json = $3, updated_at = now()
           WHERE id = $4"#,
    )
    .bind(threat_level)
    .bind(score)
    .bind(intel_json)
    .bind(node_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get IOC nodes for a given email.
pub async fn get_by_email(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<IocNodeRow>, IocError> {
    let rows = sqlx::query_as::<_, IocNodeRow>(
        r#"SELECT n.id, n.tenant_id, n.ioc_type, n.ioc_value, n.threat_level,
                  n.intel_score, n.intel_json, n.sighting_count, n.first_seen, n.last_seen
           FROM ioc_nodes n
           JOIN email_ioc_occurrences o ON o.ioc_node_id = n.id
           WHERE o.email_id = $1 AND o.tenant_id = $2"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Get a single IOC node by id.
pub async fn get_by_id(pool: &PgPool, node_id: Uuid) -> Result<Option<IocNodeRow>, IocError> {
    let row = sqlx::query_as::<_, IocNodeRow>(
        r#"SELECT id, tenant_id, ioc_type, ioc_value, threat_level,
                  intel_score, intel_json, sighting_count, first_seen, last_seen
           FROM ioc_nodes WHERE id = $1"#,
    )
    .bind(node_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Bulk get recent IOC nodes for a tenant.
pub async fn bulk_get_tenant_recent(
    pool: &PgPool,
    tenant_id: Uuid,
    since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<IocNodeRow>, IocError> {
    let rows = sqlx::query_as::<_, IocNodeRow>(
        r#"SELECT id, tenant_id, ioc_type, ioc_value, threat_level,
                  intel_score, intel_json, sighting_count, first_seen, last_seen
           FROM ioc_nodes
           WHERE tenant_id = $1 AND last_seen > $2
           ORDER BY last_seen DESC
           LIMIT 5000"#,
    )
    .bind(tenant_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
