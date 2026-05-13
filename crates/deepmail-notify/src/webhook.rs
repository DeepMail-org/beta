use crate::error::NotifyError;
use crate::pipeline::NotifyEvent;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::Duration;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_payload(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    let result = mac.finalize();
    format!("sha256={}", hex::encode(result.into_bytes()))
}

pub async fn deliver_webhook(
    client: &reqwest::Client,
    url: &str,
    secret: &str,
    event: &NotifyEvent,
    delivery_id: Uuid,
    timeout_secs: u64,
) -> Result<(), NotifyError> {
    let body = serde_json::to_vec(event)
        .map_err(|e| NotifyError::WebhookFailed(format!("serialize error: {}", e)))?;

    let signature = sign_payload(secret, &body);

    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-DeepMail-Signature", &signature)
        .header("X-DeepMail-Event", &event.event_type)
        .header("X-DeepMail-Delivery-Id", delivery_id.to_string())
        .body(body)
        .timeout(Duration::from_secs(timeout_secs))
        .send()
        .await
        .map_err(|e| NotifyError::WebhookFailed(format!("network error: {}", e)))?;

    let status = resp.status();
    if status.is_success() {
        Ok(())
    } else if status.is_client_error() {
        Err(NotifyError::WebhookFailed(format!(
            "permanent failure: HTTP {}",
            status.as_u16()
        )))
    } else {
        Err(NotifyError::WebhookFailed(format!(
            "server error: HTTP {}",
            status.as_u16()
        )))
    }
}

pub async fn deliver_with_retry(
    client: &reqwest::Client,
    url: &str,
    secret: &str,
    event: &NotifyEvent,
    delivery_id: Uuid,
    timeout_secs: u64,
) -> Result<u32, NotifyError> {
    let backoffs = [1u64, 2, 4];

    for (attempt, &delay) in backoffs.iter().enumerate() {
        match deliver_webhook(client, url, secret, event, delivery_id, timeout_secs).await {
            Ok(()) => return Ok(attempt as u32 + 1),
            Err(NotifyError::WebhookFailed(msg)) if msg.starts_with("permanent") => {
                return Err(NotifyError::WebhookFailed(msg));
            }
            Err(e) => {
                if attempt < backoffs.len() - 1 {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %e,
                        url = url,
                        "webhook delivery failed, retrying"
                    );
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                } else {
                    return Err(e);
                }
            }
        }
    }

    Err(NotifyError::WebhookFailed(
        "exhausted all retry attempts".to_string(),
    ))
}
