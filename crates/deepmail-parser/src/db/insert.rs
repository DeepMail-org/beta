//! Bulk inserts for parsed email data into the parser database.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ParserError;

/// Parsed email envelope fields.
pub struct ParsedEmailRow {
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub message_id: Option<String>,
    pub subject: Option<String>,
    pub from_address: Option<String>,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub reply_to: Option<String>,
    pub date_sent: Option<chrono::DateTime<chrono::Utc>>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachment_count: i32,
}

/// A single email header row.
pub struct HeaderRow {
    pub header_name: String,
    pub header_value: String,
    pub ordinal: i32,
}

/// A single Received: hop row.
pub struct ReceivedHopRow {
    pub hop_index: i32,
    pub from_host: Option<String>,
    pub by_host: Option<String>,
    pub for_address: Option<String>,
    pub via_protocol: Option<String>,
    pub received_at: Option<chrono::DateTime<chrono::Utc>>,
    pub raw_value: String,
}

/// A single attachment metadata row.
pub struct AttachmentRow {
    pub email_id: Uuid,
    pub tenant_id: Uuid,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: i64,
    pub sha256_hash: String,
    pub s3_bucket: String,
    pub s3_key: String,
    pub entropy: Option<f64>,
    pub ordinal: i32,
}

/// Insert the parsed_emails row. Returns the generated primary key.
pub async fn insert_parsed_email(
    pool: &PgPool,
    row: &ParsedEmailRow,
) -> Result<Uuid, ParserError> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO parsed_emails (
            email_id, tenant_id, message_id, subject,
            from_address, to_addresses, cc_addresses, bcc_addresses,
            reply_to, date_sent, body_text, body_html, attachment_count
        ) VALUES (
            $1, $2, $3, $4,
            $5, $6, $7, $8,
            $9, $10, $11, $12, $13
        )
        ON CONFLICT (email_id) DO UPDATE
            SET subject          = EXCLUDED.subject,
                from_address     = EXCLUDED.from_address,
                attachment_count = EXCLUDED.attachment_count,
                parsed_at        = now()
        RETURNING id
        "#,
        row.email_id,
        row.tenant_id,
        row.message_id,
        row.subject,
        row.from_address,
        &row.to_addresses,
        &row.cc_addresses,
        &row.bcc_addresses,
        row.reply_to,
        row.date_sent,
        row.body_text,
        row.body_html,
        row.attachment_count,
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}

/// Bulk insert email headers.
pub async fn insert_headers(
    pool: &PgPool,
    parsed_email_id: Uuid,
    headers: &[HeaderRow],
) -> Result<(), ParserError> {
    for h in headers {
        sqlx::query!(
            r#"
            INSERT INTO email_headers (parsed_email_id, header_name, header_value, ordinal)
            VALUES ($1, $2, $3, $4)
            "#,
            parsed_email_id,
            h.header_name,
            h.header_value,
            h.ordinal,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Bulk insert received hops.
pub async fn insert_received_hops(
    pool: &PgPool,
    parsed_email_id: Uuid,
    hops: &[ReceivedHopRow],
) -> Result<(), ParserError> {
    for hop in hops {
        sqlx::query!(
            r#"
            INSERT INTO received_hops (
                parsed_email_id, hop_index,
                from_host, by_host, for_address, via_protocol,
                received_at, raw_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            parsed_email_id,
            hop.hop_index,
            hop.from_host,
            hop.by_host,
            hop.for_address,
            hop.via_protocol,
            hop.received_at,
            hop.raw_value,
        )
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Insert a single attachment metadata row.
pub async fn insert_attachment(
    pool: &PgPool,
    parsed_email_id: Uuid,
    att: &AttachmentRow,
) -> Result<Uuid, ParserError> {
    let rec = sqlx::query!(
        r#"
        INSERT INTO attachments (
            parsed_email_id, email_id, tenant_id,
            filename, content_type, size_bytes,
            sha256_hash, s3_bucket, s3_key,
            entropy, ordinal
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id
        "#,
        parsed_email_id,
        att.email_id,
        att.tenant_id,
        att.filename,
        att.content_type,
        att.size_bytes,
        att.sha256_hash,
        att.s3_bucket,
        att.s3_key,
        att.entropy,
        att.ordinal,
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}
