//! gRPC service for deepmail-tenant.
//!
//! Defines a TenantService proto manually (no generated proto for tenant
//! in the current common set — we use a local proto approach here).
//!
//! The following RPCs are defined in a local tenant.proto
//! (not shared, only this service implements it):
//!   GetTenant        — lookup by tenant_id UUID
//!   GetTenantBySlug  — lookup by slug
//!   GetMemberRole    — check if user is member and get role
//!   GetSettings      — fetch tenant settings for analysis services
//!
//! These are used by the gateway and other services for authorization.

use std::sync::Arc;

use uuid::Uuid;

use crate::{config::Config, db, error::TenantError};

/// Request: look up a tenant by UUID.
#[derive(Debug)]
pub struct GetTenantRequest {
    pub tenant_id: String,
}

/// Response: tenant metadata needed by other services.
#[derive(Debug)]
pub struct TenantInfo {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub status: String,
    pub owner_user_id: String,
}

/// The gRPC service implementation struct.
/// The actual tonic trait will be wired in once the tenant.proto
/// is added to deepmail-common. For now this module contains
/// the business logic that the main.rs wires into the server.
pub struct TenantServiceHandler {
    pub pool: Arc<sqlx::PgPool>,
    pub config: Arc<Config>,
}

impl TenantServiceHandler {
    /// Get tenant by UUID. Used by gateway for JWT validation.
    #[tracing::instrument(skip(self))]
    pub async fn get_tenant(
        &self,
        tenant_id_str: &str,
    ) -> Result<db::tenants::TenantRow, TenantError> {
        let tenant_id = Uuid::parse_str(tenant_id_str)
            .map_err(|_| TenantError::Validation("invalid tenant UUID".into()))?;

        db::tenants::get_tenant_by_id(&self.pool, tenant_id)
            .await?
            .ok_or(TenantError::TenantNotFound)
    }

    /// Get a user's role in a tenant. Returns None if not a member.
    #[tracing::instrument(skip(self))]
    pub async fn get_member_role(
        &self,
        tenant_id_str: &str,
        user_id_str: &str,
    ) -> Result<Option<String>, TenantError> {
        let tenant_id = Uuid::parse_str(tenant_id_str)
            .map_err(|_| TenantError::Validation("invalid tenant UUID".into()))?;
        let user_id = Uuid::parse_str(user_id_str)
            .map_err(|_| TenantError::Validation("invalid user UUID".into()))?;

        db::members::get_member_role(&self.pool, tenant_id, user_id).await
    }

    /// Get settings for a tenant (used by analysis services).
    #[tracing::instrument(skip(self))]
    pub async fn get_settings(
        &self,
        tenant_id_str: &str,
    ) -> Result<db::settings::SettingsRow, TenantError> {
        let tenant_id = Uuid::parse_str(tenant_id_str)
            .map_err(|_| TenantError::Validation("invalid tenant UUID".into()))?;

        db::settings::get_settings(&self.pool, tenant_id)
            .await?
            .ok_or(TenantError::TenantNotFound)
    }
}
