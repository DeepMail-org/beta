use crate::error::NotifyError;
use deepmail_common::proto::auth::auth_service_client::AuthServiceClient;
use deepmail_common::proto::auth::ValidateTokenRequest;
use tonic::transport::Channel;
use uuid::Uuid;

pub struct AuthClient {
    inner: AuthServiceClient<Channel>,
}

#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role: String,
}

impl AuthClient {
    pub async fn connect(url: &str) -> Result<Self, NotifyError> {
        let channel = Channel::from_shared(url.to_string())
            .map_err(|e| NotifyError::AuthError(format!("invalid auth URL: {}", e)))?
            .connect()
            .await
            .map_err(|e| NotifyError::AuthError(format!("auth gRPC connect failed: {}", e)))?;

        Ok(Self {
            inner: AuthServiceClient::new(channel),
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<TokenClaims, NotifyError> {
        let mut client = self.inner.clone();

        let resp = client
            .validate_token(ValidateTokenRequest {
                token: token.to_string(),
            })
            .await
            .map_err(|e| NotifyError::AuthError(format!("validate_token RPC failed: {}", e)))?;

        let inner = resp.into_inner();

        if !inner.valid {
            return Err(NotifyError::AuthError("token is invalid".to_string()));
        }

        let user_id = Uuid::parse_str(&inner.user_id)
            .map_err(|e| NotifyError::AuthError(format!("invalid user_id in token: {}", e)))?;

        let tenant_id = Uuid::parse_str(&inner.tenant_id)
            .map_err(|e| NotifyError::AuthError(format!("invalid tenant_id in token: {}", e)))?;

        Ok(TokenClaims {
            user_id,
            tenant_id,
            role: inner.role,
        })
    }
}
