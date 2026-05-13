use crate::error::ReportError;
use aws_credential_types::Credentials;
use aws_sdk_s3::primitives::ByteStream;

pub fn build_s3_client(
    endpoint: &str,
    region: &str,
    access_key: &str,
    secret_key: &str,
) -> aws_sdk_s3::Client {
    let creds = Credentials::new(access_key, secret_key, None, None, "deepmail-report");

    let config = aws_sdk_s3::Config::builder()
        .region(aws_sdk_s3::config::Region::new(region.to_string()))
        .endpoint_url(endpoint)
        .credentials_provider(creds)
        .force_path_style(true)
        .behavior_version_latest()
        .build();

    aws_sdk_s3::Client::from_conf(config)
}

pub async fn upload_report(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
    bytes: Vec<u8>,
    content_type: &str,
) -> Result<String, ReportError> {
    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(ByteStream::from(bytes))
        .content_type(content_type)
        .send()
        .await
        .map_err(|e| ReportError::S3Upload(format!("{key}: {e}")))?;

    Ok(key.to_string())
}

pub async fn download_report(
    client: &aws_sdk_s3::Client,
    bucket: &str,
    key: &str,
) -> Result<Vec<u8>, ReportError> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .map_err(|e| ReportError::S3Download(format!("{key}: {e}")))?;

    let bytes = resp
        .body
        .collect()
        .await
        .map_err(|e| ReportError::S3Download(format!("{key} body: {e}")))?
        .into_bytes()
        .to_vec();

    Ok(bytes)
}
