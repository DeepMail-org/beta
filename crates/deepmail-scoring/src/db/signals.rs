use crate::error::ScoringError;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn upsert_signal(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    signal_name: &str,
    score: f32,
    raw_data: &serde_json::Value,
) -> Result<(), ScoringError> {
    sqlx::query(
        r#"INSERT INTO signal_scores (email_id, tenant_id, signal_name, score, raw_data)
           VALUES ($1, $2, $3, $4, $5)
           ON CONFLICT (email_id, signal_name) DO UPDATE SET
             score = EXCLUDED.score,
             raw_data = EXCLUDED.raw_data,
             received_at = now()"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(signal_name)
    .bind(score)
    .bind(raw_data)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_by_email(pool: &PgPool, email_id: Uuid) -> Result<i64, ScoringError> {
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM signal_scores WHERE email_id = $1")
            .bind(email_id)
            .fetch_one(pool)
            .await?;
    Ok(count.0)
}

pub async fn list_by_email(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Vec<(String, f32)>, ScoringError> {
    let rows: Vec<(String, f32)> =
        sqlx::query_as("SELECT signal_name, score FROM signal_scores WHERE email_id = $1")
            .bind(email_id)
            .fetch_all(pool)
            .await?;
    Ok(rows)
}
