use crate::error::ScoringError;
use sqlx::{PgPool, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ThreatScoreRow {
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub final_score: f32,
    pub final_verdict: String,
    pub signals_available: i32,
    pub signals_total: i32,
    pub header_score: Option<f32>,
    pub dkim_score: Option<f32>,
    pub geo_score: Option<f32>,
    pub ip_score: Option<f32>,
    pub intel_score: Option<f32>,
    pub ioc_score: Option<f32>,
    pub homograph_score: Option<f32>,
    pub body_score: Option<f32>,
    pub sandbox_url_score: Option<f32>,
    pub sandbox_file_score: Option<f32>,
    pub sandbox_dynamic_score: Option<f32>,
    pub weight_total: f32,
    pub is_final: bool,
}

pub async fn upsert_threat_score(pool: &PgPool, row: &ThreatScoreRow) -> Result<Uuid, ScoringError> {
    let rec = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO threat_scores
            (email_id, tenant_id, final_score, final_verdict,
             signals_available, signals_total,
             header_score, dkim_score, geo_score, ip_score,
             intel_score, ioc_score, homograph_score, body_score,
             sandbox_url_score, sandbox_file_score, sandbox_dynamic_score,
             weight_total, is_final, computed_at, updated_at)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,now(),now())
           ON CONFLICT (email_id) DO UPDATE SET
             final_score = EXCLUDED.final_score,
             final_verdict = EXCLUDED.final_verdict,
             signals_available = EXCLUDED.signals_available,
             signals_total = EXCLUDED.signals_total,
             header_score = EXCLUDED.header_score,
             dkim_score = EXCLUDED.dkim_score,
             geo_score = EXCLUDED.geo_score,
             ip_score = EXCLUDED.ip_score,
             intel_score = EXCLUDED.intel_score,
             ioc_score = EXCLUDED.ioc_score,
             homograph_score = EXCLUDED.homograph_score,
             body_score = EXCLUDED.body_score,
             sandbox_url_score = EXCLUDED.sandbox_url_score,
             sandbox_file_score = EXCLUDED.sandbox_file_score,
             sandbox_dynamic_score = EXCLUDED.sandbox_dynamic_score,
             weight_total = EXCLUDED.weight_total,
             is_final = EXCLUDED.is_final,
             computed_at = now(),
             updated_at = now()
           RETURNING id"#,
    )
    .bind(row.email_id)
    .bind(row.tenant_id)
    .bind(row.final_score)
    .bind(&row.final_verdict)
    .bind(row.signals_available)
    .bind(row.signals_total)
    .bind(row.header_score)
    .bind(row.dkim_score)
    .bind(row.geo_score)
    .bind(row.ip_score)
    .bind(row.intel_score)
    .bind(row.ioc_score)
    .bind(row.homograph_score)
    .bind(row.body_score)
    .bind(row.sandbox_url_score)
    .bind(row.sandbox_file_score)
    .bind(row.sandbox_dynamic_score)
    .bind(row.weight_total)
    .bind(row.is_final)
    .fetch_one(pool)
    .await?;
    Ok(rec)
}

pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<ThreatScoreRow>, ScoringError> {
    let row = sqlx::query(
        r#"SELECT email_id, tenant_id, final_score, final_verdict,
                  signals_available, signals_total,
                  header_score, dkim_score, geo_score, ip_score,
                  intel_score, ioc_score, homograph_score, body_score,
                  sandbox_url_score, sandbox_file_score, sandbox_dynamic_score,
                  weight_total, is_final
           FROM threat_scores WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| ThreatScoreRow {
        email_id: r.get("email_id"),
        tenant_id: r.get("tenant_id"),
        final_score: r.get("final_score"),
        final_verdict: r.get("final_verdict"),
        signals_available: r.get("signals_available"),
        signals_total: r.get("signals_total"),
        header_score: r.get("header_score"),
        dkim_score: r.get("dkim_score"),
        geo_score: r.get("geo_score"),
        ip_score: r.get("ip_score"),
        intel_score: r.get("intel_score"),
        ioc_score: r.get("ioc_score"),
        homograph_score: r.get("homograph_score"),
        body_score: r.get("body_score"),
        sandbox_url_score: r.get("sandbox_url_score"),
        sandbox_file_score: r.get("sandbox_file_score"),
        sandbox_dynamic_score: r.get("sandbox_dynamic_score"),
        weight_total: r.get("weight_total"),
        is_final: r.get("is_final"),
    }))
}

pub async fn mark_final(pool: &PgPool, email_id: Uuid) -> Result<(), ScoringError> {
    sqlx::query("UPDATE threat_scores SET is_final = true, updated_at = now() WHERE email_id = $1")
        .bind(email_id)
        .execute(pool)
        .await?;
    Ok(())
}
