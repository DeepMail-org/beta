use crate::error::BillingError;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct LineItem {
    pub name: String,
    pub amount_paise: i64,
    pub quantity: u32,
}

#[derive(Debug, serde::Deserialize)]
pub struct RazorpayInvoice {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub amount: i64,
}

pub struct RazorpayClient {
    client: reqwest::Client,
    key_id: String,
    key_secret: String,
    base_url: String,
}

impl RazorpayClient {
    pub fn new(key_id: String, key_secret: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            key_id,
            key_secret,
            base_url: "https://api.razorpay.com/v1".to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.key_id.is_empty() && !self.key_secret.is_empty()
    }

    pub async fn create_invoice(
        &self,
        tenant_name: &str,
        billing_email: &str,
        line_items: &[LineItem],
        period: &str,
    ) -> Result<String, BillingError> {
        if !self.is_configured() {
            return Err(BillingError::NotConfigured);
        }

        let items: Vec<serde_json::Value> = line_items
            .iter()
            .map(|li| {
                serde_json::json!({
                    "name": li.name,
                    "amount": li.amount_paise,
                    "quantity": li.quantity,
                })
            })
            .collect();

        let body = serde_json::json!({
            "type": "invoice",
            "description": format!("DeepMail billing {}", period),
            "customer": {
                "name": tenant_name,
                "email": billing_email,
            },
            "line_items": items,
        });

        let resp = self
            .client
            .post(format!("{}/invoices", self.base_url))
            .basic_auth(&self.key_id, Some(&self.key_secret))
            .json(&body)
            .send()
            .await
            .map_err(|e| BillingError::RazorpayError(format!("network error: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(BillingError::RazorpayError(format!(
                "create invoice failed: HTTP {} — {}",
                status.as_u16(),
                text
            )));
        }

        let invoice: RazorpayInvoice = resp
            .json()
            .await
            .map_err(|e| BillingError::RazorpayError(format!("parse response: {}", e)))?;

        Ok(invoice.id)
    }

    pub async fn issue_invoice(&self, invoice_id: &str) -> Result<(), BillingError> {
        if !self.is_configured() {
            return Err(BillingError::NotConfigured);
        }

        let resp = self
            .client
            .post(format!("{}/invoices/{}/issue", self.base_url, invoice_id))
            .basic_auth(&self.key_id, Some(&self.key_secret))
            .send()
            .await
            .map_err(|e| BillingError::RazorpayError(format!("network error: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(BillingError::RazorpayError(format!(
                "issue invoice failed: HTTP {} — {}",
                status.as_u16(),
                text
            )));
        }

        Ok(())
    }

    pub async fn get_invoice(&self, invoice_id: &str) -> Result<RazorpayInvoice, BillingError> {
        if !self.is_configured() {
            return Err(BillingError::NotConfigured);
        }

        let resp = self
            .client
            .get(format!("{}/invoices/{}", self.base_url, invoice_id))
            .basic_auth(&self.key_id, Some(&self.key_secret))
            .send()
            .await
            .map_err(|e| BillingError::RazorpayError(format!("network error: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(BillingError::RazorpayError(format!(
                "get invoice failed: HTTP {} — {}",
                status.as_u16(),
                text
            )));
        }

        resp.json::<RazorpayInvoice>()
            .await
            .map_err(|e| BillingError::RazorpayError(format!("parse response: {}", e)))
    }

    pub fn verify_webhook_signature(payload: &[u8], signature: &str, webhook_secret: &str) -> bool {
        let mut mac = match HmacSha256::new_from_slice(webhook_secret.as_bytes()) {
            Ok(m) => m,
            Err(_) => return false,
        };
        mac.update(payload);
        let expected = hex::encode(mac.finalize().into_bytes());
        expected == signature
    }
}
