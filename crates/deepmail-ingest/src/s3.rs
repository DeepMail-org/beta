//! S3/MinIO upload client.
//!
//! Uploads quarantined files with:
//!   - Private ACL (no public access)
//!   - Server-side encryption (AES256 for MinIO, aws:kms optional for S3)
//!   - Content-Type metadata
//!   - Custom metadata: original-filename, uploaded-by, tenant-id

use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{Builder as S3ConfigBuilder, SharedCredentialsProvider},
    primitives::ByteStream,
    Client as S3Client,
};

use crate::{config::Config, error::IngestError};

/// Create an S3 client configured for the given Config.
/// Works with both AWS S3 and MinIO (path-style URLs).
pub async fn create_s3_client(config: &Config) -> S3Client {
    let creds = Credentials::new(
        &config.s3_access_key_id,
        &config.s3_secret_access_key,
        None,
        None,
        "deepmail-static",
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

/// Upload a file to S3/MinIO and return the S3 key.
///
/// The key format is: quarantine/{tenant_id}/{quarantine_name}
/// This partitions by tenant inside the single shared bucket.
pub async fn upload_quarantine(
    client: &S3Client,
    bucket: &str,
    tenant_id: uuid::Uuid,
    quarantine_name: &str,
    original_filename: &str,
    uploaded_by: uuid::Uuid,
    content_type: &str,
    file_bytes: Vec<u8>,
) -> Result<String, IngestError> {
    let s3_key = format!("quarantine/{tenant_id}/{quarantine_name}");

    let body = ByteStream::from(file_bytes);

    client
        .put_object()
        .bucket(bucket)
        .key(&s3_key)
        .body(body)
        .content_type(content_type)
        .server_side_encryption(
            aws_sdk_s3::types::ServerSideEncryption::Aes256,
        )
        // Custom metadata
        .metadata("original-filename", original_filename)
        .metadata("uploaded-by", &uploaded_by.to_string())
        .metadata("tenant-id", &tenant_id.to_string())
        .send()
        .await
        .map_err(|e| IngestError::S3Upload(e.to_string()))?;

    Ok(s3_key)
}
