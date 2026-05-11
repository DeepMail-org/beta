use crate::error::ScoringError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn insert_feedback(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    analyst_id: Uuid,
    predicted_verdict: &str,
    correct_verdict: &str,
    feedback_notes: Option<&str>,
    signal_overrides: &serde_json::Value,
) -> Result<Uuid, ScoringError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO analyst_feedback
            (email_id, tenant_id, analyst_id, predicted_verdict,
             correct_verdict, feedback_notes, signal_overrides)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING id"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(analyst_id)
    .bind(predicted_verdict)
    .bind(correct_verdict)
    .bind(feedback_notes)
    .bind(signal_overrides)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn list_by_email(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<(Uuid, String, String)>, ScoringError> {
    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, predicted_verdict, correct_verdict FROM analyst_feedback WHERE email_id = $1 ORDER BY created_at DESC",
    )
    .bind(email_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}
