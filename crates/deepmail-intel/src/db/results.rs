/// DB operations for email_intel_results table.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::IntelError;

pub async fn upsert_email_result(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    iocs_analyzed: i32,
    max_vt_score: f32,
    malicious_iocs: i32,
    provider_hits: &[String],
    summary_json: &serde_json::Value,
) -> Result<(), IntelError> {
    sqlx::query(
        r#"INSERT INTO email_intel_results
               (email_id, tenant_id, iocs_analyzed, max_vt_score,
                malicious_iocs, provider_hits, summary_json)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (email_id) DO UPDATE SET
               iocs_analyzed  = EXCLUDED.iocs_analyzed,
               max_vt_score   = EXCLUDED.max_vt_score,
               malicious_iocs = EXCLUDED.malicious_iocs,
               provider_hits  = EXCLUDED.provider_hits,
               summary_json   = EXCLUDED.summary_json,
               analyzed_at    = now()"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(iocs_analyzed)
    .bind(max_vt_score)
    .bind(malicious_iocs)
    .bind(provider_hits)
    .bind(summary_json)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<EmailIntelRow>, IntelError> {
    let row = sqlx::query_as::<_, EmailIntelRow>(
        r#"SELECT id, email_id, tenant_id, iocs_analyzed, max_vt_score,
                  malicious_iocs, provider_hits, summary_json, analyzed_at, created_at
           FROM email_intel_results
           WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmailIntelRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub iocs_analyzed: i32,
    pub max_vt_score: f32,
    pub malicious_iocs: i32,
    pub provider_hits: Vec<String>,
    pub summary_json: serde_json::Value,
    pub analyzed_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
