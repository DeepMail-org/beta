//! Error types for deepmail-tenant.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TenantError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("tenant not found")]
    TenantNotFound,

    #[error("tenant already exists with slug: {0}")]
    TenantAlreadyExists(String),

    #[error("member not found")]
    MemberNotFound,

    #[error("user is already a member of this tenant")]
    AlreadyMember,

    #[error("invite not found or expired")]
    InviteNotFound,

    #[error("invite already accepted")]
    InviteAlreadyAccepted,

    #[error("invalid Razorpay webhook signature")]
    InvalidWebhookSignature,

    #[error("Razorpay event already processed: {0}")]
    DuplicateWebhookEvent(String),

    #[error("subscription not found")]
    SubscriptionNotFound,

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("input validation failed: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<TenantError> for tonic::Status {
    fn from(e: TenantError) -> Self {
        match e {
            TenantError::Database(ref inner) => {
                tracing::error!(error = %inner, "database error in tenant service");
                tonic::Status::internal("internal server error")
            }
            TenantError::TenantNotFound | TenantError::MemberNotFound => {
                tonic::Status::not_found(e.to_string())
            }
            TenantError::TenantAlreadyExists(_) | TenantError::AlreadyMember => {
                tonic::Status::already_exists(e.to_string())
            }
            TenantError::InviteNotFound | TenantError::InviteAlreadyAccepted => {
                tonic::Status::not_found(e.to_string())
            }
            TenantError::InvalidWebhookSignature => {
                tonic::Status::unauthenticated("invalid webhook signature")
            }
            TenantError::DuplicateWebhookEvent(_) => {
                tonic::Status::already_exists(e.to_string())
            }
            TenantError::SubscriptionNotFound => {
                tonic::Status::not_found("subscription not found")
            }
            TenantError::PermissionDenied(ref msg) => {
                tonic::Status::permission_denied(msg.clone())
            }
            TenantError::Validation(ref msg) => {
                tonic::Status::invalid_argument(msg.clone())
            }
            TenantError::Internal(ref msg) => {
                tracing::error!(error = %msg, "internal error in tenant service");
                tonic::Status::internal("internal server error")
            }
        }
    }
}
