//! gRPC service implementation for `AuthService`.
//!
//! Implements the three RPCs defined in `auth.proto`:
//!   - `ValidateToken`
//!   - `GetUserById`
//!   - `RevokeToken`

use std::sync::Arc;

use tonic::{Request, Response, Status};
use uuid::Uuid;

use deepmail_common::proto::auth::{
    auth_service_server::AuthService, GetUserRequest, RevokeTokenRequest,
    RevokeTokenResponse, UserResponse, ValidateTokenRequest,
    ValidateTokenResponse,
};

use crate::{config::Config, crypto::jwt::JwtManager, db, error::AuthError};

/// Convert a `chrono::DateTime<Utc>` to a `prost_types::Timestamp`.
fn to_proto_timestamp(
    dt: chrono::DateTime<chrono::Utc>,
) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

/// gRPC `AuthService` implementation.
pub struct AuthServiceImpl {
    pub pool: Arc<sqlx::PgPool>,
    pub jwt: Arc<JwtManager>,
    pub config: Arc<Config>,
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    /// Validate a JWT access token.
    ///
    /// Returns `valid=false` (not an error) for invalid/expired tokens, so
    /// the gateway can distinguish auth failures from infrastructure errors.
    #[tracing::instrument(skip(self, request))]
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let token = request.into_inner().token;

        match self.jwt.validate_token(&token) {
            Ok(claims) => {
                let expires_at =
                    chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                        .map(to_proto_timestamp);

                Ok(Response::new(ValidateTokenResponse {
                    valid: true,
                    user_id: claims.sub,
                    tenant_id: claims.tenant_id,
                    role: claims.role,
                    jti: claims.jti,
                    expires_at,
                }))
            }
            Err(AuthError::TokenExpired | AuthError::TokenInvalid) => {
                Ok(Response::new(ValidateTokenResponse {
                    valid: false,
                    user_id: String::new(),
                    tenant_id: String::new(),
                    role: String::new(),
                    jti: String::new(),
                    expires_at: None,
                }))
            }
            Err(e) => Err(Status::from(e)),
        }
    }

    /// Fetch a user by their UUID.
    #[tracing::instrument(skip(self, request))]
    async fn get_user_by_id(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<UserResponse>, Status> {
        let user_id_str = request.into_inner().user_id;
        let user_id = Uuid::parse_str(&user_id_str).map_err(|_| {
            Status::invalid_argument("invalid user_id UUID format")
        })?;

        let user = db::users::get_user_by_id(&self.pool, user_id)
            .await
            .map_err(Status::from)?
            .ok_or_else(|| Status::not_found("user not found"))?;

        let last_login_at = user.last_login_at.map(to_proto_timestamp);

        Ok(Response::new(UserResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            // Default — the tenant service is the authoritative role source.
            role: String::from("analyst"),
            is_active: user.is_active,
            email_verified: user.email_verified,
            created_at: Some(to_proto_timestamp(user.created_at)),
            last_login_at,
        }))
    }

    /// Revoke a refresh token or JWT by JTI.
    ///
    /// Currently logs the revocation. The gateway maintains a Redis set of
    /// revoked JTIs with TTL — full persistence is wired up in that session.
    #[tracing::instrument(skip(self, request))]
    async fn revoke_token(
        &self,
        request: Request<RevokeTokenRequest>,
    ) -> Result<Response<RevokeTokenResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(
            jti = %req.jti,
            reason = %req.reason,
            "token revocation requested"
        );
        Ok(Response::new(RevokeTokenResponse { revoked: true }))
    }
}
