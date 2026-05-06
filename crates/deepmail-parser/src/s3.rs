//! S3 helpers for fetching quarantined emails and storing attachments.

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{Builder as S3ConfigBuilder, SharedCredentialsProvider},
    primitives::ByteStream,
    Client as S3Client,
};

use crate::{config::Config, error::ParserError};

/// Create an S3 client configured for the given Config.
pub async fn create_s3_client(config: &Config) -> S3Client {
    let creds = Credentials::new(
        &config.s3_access_key_id,
        &config.s3_secret_access_key,
        None,
        None,
        "deepmail-parser-static",
    );

    let s3_config = S3ConfigBuilder::new()
        .region(Region::new(config.s3_region.clone()))
        .endpoint_url(&config.s3_endpoint)
        .credentials_provider(SharedCredentialsProvider::new(creds))
        .force_path_style(config.s3_force_path_style)
        .behavior_version_latest()
        .build();

    S3Client::from_conf(s3_config)
}

/// Fetch the quarantined email bytes from S3.
pub async fn fetch_quarantine(
    client: &S3Client,
    bucket: &str,
    s3_key: &str,
) -> Result<Vec<u8>, ParserError> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(s3_key)
        .send()
        .await
        .map_err(|e| ParserError::S3(format!("get {s3_key}: {e}")))?;

    let body = resp
        .body
        .collect()
        .await
        .map_err(|e| ParserError::S3(format!("read body {s3_key}: {e}")))?;

    Ok(body.into_bytes().to_vec())
}

/// Upload an attachment blob to S3 with AES-256 encryption.
///
/// Key format: `attachments/{tenant_id}/{email_id}/{ordinal}_{filename}`
pub async fn upload_attachment(
    client: &S3Client,
    bucket: &str,
    tenant_id: uuid::Uuid,
    email_id: uuid::Uuid,
    ordinal: i32,
    filename: &str,
    content_type: &str,
    data: Vec<u8>,
) -> Result<String, ParserError> {
    let safe_name = filename.replace(['/', '\\', '\0'], "_");
    let s3_key = format!("attachments/{tenant_id}/{email_id}/{ordinal}_{safe_name}");

    let body = ByteStream::from(data);

    client
        .put_object()
        .bucket(bucket)
        .key(&s3_key)
        .body(body)
        .content_type(content_type)
        .server_side_encryption(aws_sdk_s3::types::ServerSideEncryption::Aes256)
        .metadata("tenant-id", &tenant_id.to_string())
        .metadata("email-id", &email_id.to_string())
        .metadata("original-filename", filename)
        .send()
        .await
        .map_err(|e| ParserError::S3(format!("put attachment {s3_key}: {e}")))?;

    Ok(s3_key)
}
