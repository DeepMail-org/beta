/// S3 (MinIO) attachment download helper.

use crate::error::SandboxFileError;

/// Download an attachment from S3/MinIO and return raw bytes.
pub async fn download_attachment(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, SandboxFileError> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| SandboxFileError::S3(format!("get_object {}: {}", key, e)))?;

    let body = resp
        .body
        .collect()
        .await
        .map_err(|e| SandboxFileError::S3(format!("collect body: {}", e)))?;

    Ok(body.into_bytes().to_vec())
}
