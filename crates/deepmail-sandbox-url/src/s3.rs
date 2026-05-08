/// S3 (MinIO) screenshot upload helper.

use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::primitives::ByteStream;

use crate::error::SandboxUrlError;

/// Upload a screenshot PNG to S3 and return the key.
pub async fn upload_screenshot(
    client: &S3Client,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
) -> Result<String, SandboxUrlError> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(bytes))
        .content_type("image/png")
        .send()
        .await
        .map_err(|e| SandboxUrlError::S3(format!("upload screenshot: {}", e)))?;

    tracing::debug!(key = %key, "screenshot uploaded to S3");
    Ok(key.to_string())
}
