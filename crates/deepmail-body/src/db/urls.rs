/// Extracted URL database operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::BodyError;
use crate::urls::ExtractedUrl;

/// Bulk insert extracted URLs in batches of 100.
pub async fn bulk_insert_urls(
    pool: &PgPool,
    analysis_id: Uuid,
    urls: &[ExtractedUrl],
) -> Result<(), BodyError> {
    for chunk in urls.chunks(100) {
        for u in chunk {
            sqlx::query(
                r#"INSERT INTO extracted_urls
                     (analysis_id, raw_url, normalized_url, url_type,
                      is_shortened, shortener_domain, is_external,
                      destination_domain, is_suspicious, sent_to_sandbox)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                   ON CONFLICT DO NOTHING"#,
            )
            .bind(analysis_id)
            .bind(&u.raw_url)
            .bind(&u.normalized_url)
            .bind(&u.url_type)
            .bind(u.is_shortened)
            .bind(&u.shortener_domain)
            .bind(u.is_external)
            .bind(&u.destination_domain)
            .bind(u.is_suspicious)
            .bind(false) // sent_to_sandbox
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

/// Mark a URL as sent to sandbox.
pub async fn mark_sent_to_sandbox(
    pool: &PgPool,
    analysis_id: Uuid,
    normalized_url: &str,
) -> Result<(), BodyError> {
    sqlx::query(
        "UPDATE extracted_urls SET sent_to_sandbox = true
         WHERE analysis_id = $1 AND normalized_url = $2",
    )
    .bind(analysis_id)
    .bind(normalized_url)
    .execute(pool)
    .await?;
    Ok(())
}
