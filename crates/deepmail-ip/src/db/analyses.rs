/// email_ip_analyses table operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IpError;

#[derive(Debug, Clone)]
pub struct EmailIpAnalysisRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub max_threat_score: f32,
    pub max_verdict: String,
    pub summary_json: serde_json::Value,
}

/// Get existing analysis for an email (idempotency check).
pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<EmailIpAnalysisRow>, IpError> {
    let row = sqlx::query(
        r#"SELECT id, email_id, tenant_id, max_threat_score, max_verdict, summary_json
           FROM email_ip_analyses
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            use sqlx::Row;
            Ok(Some(EmailIpAnalysisRow {
                id: r.get("id"),
                email_id: r.get("email_id"),
                tenant_id: r.get("tenant_id"),
                max_threat_score: r.get("max_threat_score"),
                max_verdict: r.get("max_verdict"),
                summary_json: r.get("summary_json"),
            }))
        }
        None => Ok(None),
    }
}

/// Upsert email IP analysis results.
pub async fn upsert_email_analysis(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    analyzed_ips: &[String],
    max_threat_score: f32,
    max_verdict: &str,
    summary_json: &serde_json::Value,
) -> Result<Uuid, IpError> {
    let row = sqlx::query(
        r#"INSERT INTO email_ip_analyses (
               email_id, tenant_id, analyzed_ips,
               max_threat_score, max_verdict, summary_json
           )
           VALUES ($1, $2, $3::inet[], $4, $5, $6)
           ON CONFLICT (email_id)
           DO UPDATE SET
               max_threat_score = EXCLUDED.max_threat_score,
               max_verdict = EXCLUDED.max_verdict,
               summary_json = EXCLUDED.summary_json,
               analyzed_at = now()
           RETURNING id"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(analyzed_ips)
    .bind(max_threat_score)
    .bind(max_verdict)
    .bind(summary_json)
    .fetch_one(pool)
    .await?;

    use sqlx::Row;
    Ok(row.get("id"))
}
