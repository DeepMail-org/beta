//! CRUD for the hash_clusters table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::HashDbError;

/// Insert a fuzzy hash cluster relationship.
/// Uses ON CONFLICT DO NOTHING to safely handle duplicate pairs.
pub async fn insert_cluster(
    pool: &PgPool,
    representative_id: Uuid,
    cluster_member_id: Uuid,
    similarity_pct: i32,
    method: &str,
) -> Result<(), HashDbError> {
    sqlx::query!(
        r#"
        INSERT INTO hash_clusters
          (representative_hash_id, cluster_hash_id, similarity_pct, method)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (representative_hash_id, cluster_hash_id, method)
        DO NOTHING
        "#,
        representative_id,
        cluster_member_id,
        similarity_pct,
        method,
    )
    .execute(pool)
    .await
    .map_err(HashDbError::Database)?;
    Ok(())
}
