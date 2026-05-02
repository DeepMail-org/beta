//! Database access layer for the `audit_logs` table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Insert an audit log entry.
///
/// Audit log failures must not break the main auth flow — callers
/// should treat errors here as non-fatal and log/continue.
#[allow(clippy::too_many_arguments)]
pub async fn insert_audit_log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: Option<&str>,
    resource_id: Option<Uuid>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    metadata: serde_json::Value
) -> Result<(), AuthError> {
    let ip: Option<std::net::IpAddr> = ip_address.and_then(|s| s.parse().ok());

    sqlx::query!(
            r#"
        INSERT INTO audit_logs
          (user_id, action, resource_type, resource_id,
           ip_address, user_agent, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
            user_id,
            action,
            resource_type,
            resource_id,
            ip as Option<std::net::IpAddr>,
            user_agent,
            metadata
        )
        .execute(pool).await
        .map_err(AuthError::Database)?;

    Ok(())
}
