/// S3 / MinIO helpers for downloading attachments and uploading reports.

use crate::error::DynamicError;

/// Download an attachment from MinIO/S3.
pub async fn download_attachment(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, DynamicError> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| DynamicError::S3(format!("download {}: {}", key, e)))?;

    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| DynamicError::S3(format!("read body {}: {}", key, e)))?
        .into_bytes();

    Ok(bytes.to_vec())
}

/// Upload raw CAPE report to MinIO/S3.
pub async fn upload_report(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    data: &[u8],
) -> Result<(), DynamicError> {
    let body = aws_sdk_s3::primitives::ByteStream::from(data.to_vec());

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .content_type("application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| DynamicError::S3(format!("upload {}: {}", key, e)))?;

    Ok(())
}
