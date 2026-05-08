/// Body analysis DB operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::BodyError;

/// Row from body_analyses.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BodyAnalysisRow {
    pub id: Uuid,
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub plain_text_length: i32,
    pub html_length: i32,
    pub url_count: i32,
    pub external_url_count: i32,
    pub shortener_url_count: i32,
    pub qr_code_count: i32,
    pub base64_url_count: i32,
    pub keyword_score: f32,
    pub ml_score: Option<f32>,
    pub final_phishing_score: f32,
    pub has_obfuscation: bool,
    pub has_tracking_pixels: bool,
    pub has_c2_beacons: bool,
    pub obfuscation_types: Vec<String>,
    pub urgency_score: f32,
    pub verdict: String,
}

/// Get existing analysis by email_id (idempotency check).
pub async fn get_by_email_id(
    pool: &PgPool,
    email_id: Uuid,
) -> Result<Option<BodyAnalysisRow>, BodyError> {
    let row = sqlx::query_as::<_, BodyAnalysisRow>(
        r#"SELECT id, email_id, tenant_id, plain_text_length, html_length,
                  url_count, external_url_count, shortener_url_count,
                  qr_code_count, base64_url_count, keyword_score, ml_score,
                  final_phishing_score, has_obfuscation, has_tracking_pixels,
                  has_c2_beacons, obfuscation_types, urgency_score, verdict
           FROM body_analyses WHERE email_id = $1"#,
    )
    .bind(email_id)
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

/// Upsert a body analysis record. Returns the analysis id.
#[allow(clippy::too_many_arguments)]
pub async fn upsert_analysis(
    pool: &PgPool,
    email_id: Uuid,
    tenant_id: Uuid,
    plain_text_length: i32,
    html_length: i32,
    url_count: i32,
    external_url_count: i32,
    shortener_url_count: i32,
    qr_code_count: i32,
    base64_url_count: i32,
    keyword_score: f32,
    ml_score: Option<f32>,
    final_phishing_score: f32,
    has_obfuscation: bool,
    has_tracking_pixels: bool,
    has_c2_beacons: bool,
    obfuscation_types: &[String],
    urgency_score: f32,
    verdict: &str,
) -> Result<Uuid, BodyError> {
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO body_analyses
             (email_id, tenant_id, plain_text_length, html_length,
              url_count, external_url_count, shortener_url_count,
              qr_code_count, base64_url_count, keyword_score, ml_score,
              final_phishing_score, has_obfuscation, has_tracking_pixels,
              has_c2_beacons, obfuscation_types, urgency_score, verdict)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
           ON CONFLICT (email_id) DO UPDATE
             SET plain_text_length = EXCLUDED.plain_text_length,
                 html_length = EXCLUDED.html_length,
                 url_count = EXCLUDED.url_count,
                 external_url_count = EXCLUDED.external_url_count,
                 shortener_url_count = EXCLUDED.shortener_url_count,
                 qr_code_count = EXCLUDED.qr_code_count,
                 base64_url_count = EXCLUDED.base64_url_count,
                 keyword_score = EXCLUDED.keyword_score,
                 ml_score = EXCLUDED.ml_score,
                 final_phishing_score = EXCLUDED.final_phishing_score,
                 has_obfuscation = EXCLUDED.has_obfuscation,
                 has_tracking_pixels = EXCLUDED.has_tracking_pixels,
                 has_c2_beacons = EXCLUDED.has_c2_beacons,
                 obfuscation_types = EXCLUDED.obfuscation_types,
                 urgency_score = EXCLUDED.urgency_score,
                 verdict = EXCLUDED.verdict,
                 analyzed_at = now()
           RETURNING id"#,
    )
    .bind(email_id)
    .bind(tenant_id)
    .bind(plain_text_length)
    .bind(html_length)
    .bind(url_count)
    .bind(external_url_count)
    .bind(shortener_url_count)
    .bind(qr_code_count)
    .bind(base64_url_count)
    .bind(keyword_score)
    .bind(ml_score)
    .bind(final_phishing_score)
    .bind(has_obfuscation)
    .bind(has_tracking_pixels)
    .bind(has_c2_beacons)
    .bind(obfuscation_types)
    .bind(urgency_score)
    .bind(verdict)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}
