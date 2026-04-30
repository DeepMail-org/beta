//! NATS JetStream helpers and the standard message envelope used by every
//! DeepMail service.
//!
//! All inter-service async work flows over JetStream. Subjects are namespaced
//! under `deepmail.*` and split into job streams (consumed by workers) and
//! event streams (broadcast for fan-out).

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DeepMailError;

/// All NATS JetStream subject constants.
pub mod subjects {
    /// New raw email uploaded; ingest workers fan out from here.
    pub const JOBS_INGEST: &str = "deepmail.jobs.ingest";
    /// Parser pipeline input: raw email bytes ready to be RFC-5322 parsed.
    pub const JOBS_PARSE: &str = "deepmail.jobs.parse";
    /// Analyzer pipeline input: parsed email ready for the threat fan-out.
    pub const JOBS_ANALYSIS: &str = "deepmail.jobs.analysis";
    /// Per-analyzer result events (consumed by scoring + graph).
    pub const EVENTS_RESULTS: &str = "deepmail.events.results";
    /// High-severity alerts (consumed by notify).
    pub const EVENTS_ALERTS: &str = "deepmail.events.alerts";
    /// Billing / metering events (consumed by billing).
    pub const EVENTS_BILLING: &str = "deepmail.events.billing";
    /// Sandbox lifecycle events (queued / running / completed).
    pub const EVENTS_SANDBOX: &str = "deepmail.events.sandbox";
}

/// Standard envelope wrapping every NATS message payload.
///
/// Every analyzer reads its tenant + email + trace context from the envelope
/// rather than re-deriving it from the per-service payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsEnvelope {
    /// The email being processed.
    pub email_id: Uuid,
    /// The tenant who uploaded this email.
    pub tenant_id: Uuid,
    /// The user who uploaded this email.
    pub user_id: Uuid,
    /// OpenTelemetry trace ID for distributed tracing.
    pub trace_id: String,
    /// ISO 8601 timestamp when this event was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Service-specific payload.
    pub payload: serde_json::Value,
}

impl NatsEnvelope {
    /// Build a fresh envelope. `created_at` is set to "now".
    pub fn new(
        email_id: Uuid,
        tenant_id: Uuid,
        user_id: Uuid,
        trace_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            email_id,
            tenant_id,
            user_id,
            trace_id: trace_id.into(),
            created_at: chrono::Utc::now(),
            payload,
        }
    }

    /// Encode this envelope as JSON ready for [`async_nats`] publishing.
    pub fn to_bytes(&self) -> Result<Bytes, DeepMailError> {
        let v = serde_json::to_vec(self)?;
        Ok(Bytes::from(v))
    }

    /// Decode a JSON-encoded envelope.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeepMailError> {
        let env = serde_json::from_slice::<Self>(bytes)?;
        Ok(env)
    }
}

/// Connect to NATS and return a JetStream context.
///
/// `nats_url` example: `nats://nats.deepmail.svc:4222`. TLS, credentials, and
/// JWT auth should be encoded into the URL or provided via env (the underlying
/// async-nats client honours `NATS_*` variables).
pub async fn create_jetstream(
    nats_url: &str,
) -> Result<async_nats::jetstream::Context, DeepMailError> {
    let client = async_nats::connect(nats_url).await?;
    let js = async_nats::jetstream::new(client);
    tracing::info!(url = nats_url, "jetstream context created");
    Ok(js)
}

/// Publish a message to a NATS subject with the standard envelope.
///
/// Returns once the JetStream broker has acknowledged the publish.
pub async fn publish_envelope(
    js: &async_nats::jetstream::Context,
    subject: &str,
    envelope: &NatsEnvelope,
) -> Result<(), DeepMailError> {
    let payload = envelope.to_bytes()?;
    let ack = js
        .publish(subject.to_string(), payload)
        .await?;
    let _ = ack.await?;
    Ok(())
}
