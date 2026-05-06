//! Parser-specific error type.
//!
//! Every variant carries the `is_recoverable` flag so the consumer loop
//! can decide between ACK (permanent failure → drop) and NAK (transient
//! failure → retry).

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    /// Envelope payload could not be decoded (bad JSON, missing fields).
    #[error("envelope decode error: {0}")]
    EnvelopeDecode(String),

    /// The email bytes could not be parsed as RFC 5322 / MIME.
    #[error("email parse error: {0}")]
    EmailParse(String),

    /// S3 operation failed (get or put).
    #[error("s3 error: {0}")]
    S3(String),

    /// PostgreSQL error (insert, update, connection).
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// NATS publish failed.
    #[error("nats publish error: {0}")]
    NatsPublish(String),

    /// Attachment hash / entropy computation failed.
    #[error("attachment processing error: {0}")]
    AttachmentProcessing(String),

    /// Generic internal failure.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ParserError {
    /// Whether the caller should NAK (retry) vs ACK (discard).
    ///
    /// Recoverable = transient infrastructure issue; the message is
    /// structurally valid and may succeed on a subsequent attempt.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ParserError::S3(_)
                | ParserError::Database(_)
                | ParserError::NatsPublish(_)
                | ParserError::Internal(_)
        )
    }
}

impl From<deepmail_common::error::DeepMailError> for ParserError {
    fn from(err: deepmail_common::error::DeepMailError) -> Self {
        match err {
            deepmail_common::error::DeepMailError::Database(e) => {
                ParserError::Database(e)
            }
            deepmail_common::error::DeepMailError::Nats(e)
            | deepmail_common::error::DeepMailError::JetStream(e) => {
                ParserError::NatsPublish(e)
            }
            other => ParserError::Internal(other.to_string()),
        }
    }
}
