/// Homograph analysis database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::HomographError;

/// Row from homograph_analyses.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalysisRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub domains_checked: i32,
    pub high_risk_count: i32,
    pub overall_risk: String,
}

/// Check if analysis already exists for this email (idempotency).
pub async fn get_by_email(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<AnalysisRow>, HomographError> {
    let row = sqlx::query_as::<_, AnalysisRow>(
        r#"SELECT id, email_id, tenant_id, domains_checked, high_risk_count, overall_risk
           FROM homograph_analyses WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Insert a new analysis record. Returns the analysis id.
pub async fn insert_analysis(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    domains_checked: i32,
    high_risk_count: i32,
    overall_risk: &str,
) -> Result<Uuid, HomographError> {
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO homograph_analyses
             (email_id, tenant_id, domains_checked, high_risk_count, overall_risk)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (email_id) DO UPDATE
             SET domains_checked = EXCLUDED.domains_checked,
                 high_risk_count = EXCLUDED.high_risk_count,
                 overall_risk = EXCLUDED.overall_risk,
                 analyzed_at = now()
           RETURNING id"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(domains_checked)
    .bind(high_risk_count)
    .bind(overall_risk)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
