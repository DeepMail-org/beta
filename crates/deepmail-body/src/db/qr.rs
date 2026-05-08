/// QR code finding database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::BodyError;
use crate::html::QrFinding;

/// Insert a QR code finding record.
pub async fn insert_qr_finding(
    pool: &PgPool,
    analysis_id: Uuid,
    finding: &QrFinding,
) -> Result<(), BodyError> {
    sqlx::query(
        r#"INSERT INTO qr_code_findings
             (analysis_id, image_src, image_type, width, height, alt_text)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(analysis_id)
    .bind(&finding.image_src)
    .bind(&finding.image_type)
    .bind(finding.width)
    .bind(finding.height)
    .bind(&finding.alt_text)
    .execute(pool)
    .await?;

    Ok(())
}
