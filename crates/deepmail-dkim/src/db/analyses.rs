use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::DkimError;
use crate::signals::SignalScores;

pub struct AnalysisRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub raw_dkim_headers: Vec<String>,
    pub overall_verdict: String,
    pub replay_confidence: f32,
    pub signals_json: serde_json::Value,
    pub analyzed_at: DateTime<Utc>,
}

pub async fn upsert_analysis(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    raw_dkim_headers: &[String],
    overall_verdict: &str,
    replay_confidence: f32,
    signals: &SignalScores,
) -> Result<Uuid, DkimError> {
    let signals_json = serde_json::to_value(signals)
        .map_err(|e| DkimError::Internal(format!("serialize signals: {e}")))?;

    let id = sqlx::query_scalar!(
        r#"INSERT INTO dkim_analyses (
             email_id, tenant_id, raw_dkim_headers,
             overall_verdict, replay_confidence, signals_json
           ) VALUES ($1, $2, $3, $4, $5, $6)
           ON CONFLICT (email_id) DO UPDATE SET
             raw_dkim_headers  = EXCLUDED.raw_dkim_headers,
             overall_verdict   = EXCLUDED.overall_verdict,
             replay_confidence = EXCLUDED.replay_confidence,
             signals_json      = EXCLUDED.signals_json,
             analyzed_at       = now()
           RETURNING id"#,
        email_id,
        tenant_id,
        raw_dkim_headers,
        overall_verdict,
        replay_confidence,
        signals_json,
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<AnalysisRow>, DkimError> {
    let row = sqlx::query_as!(
        AnalysisRow,
        r#"SELECT
             id, email_id, tenant_id,
             raw_dkim_headers, overall_verdict,
             replay_confidence, signals_json,
             analyzed_at
           FROM dkim_analyses
           WHERE email_id = $1"#,
        email_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}
