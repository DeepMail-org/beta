//! JWT issuance and validation using RS256 (RSA + SHA-256).
//!
//! Access tokens expire in 15 minutes (configurable). Each token has a
//! unique `jti` (UUID v4) for revocation tracking. The gateway calls
//! `validate_token` on every authenticated request.

use chrono::Utc;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use uuid::Uuid;

use crate::error::AuthError;

/// JWT claims embedded in every DeepMail access token.
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Claims {
    /// Subject — the user's UUID as a string.
    pub sub: String,
    /// Tenant the user belongs to.
    pub tenant_id: String,
    /// User's RBAC role: "owner" | "admin" | "analyst" | "viewer".
    pub role: String,
    /// Expiration timestamp (Unix epoch seconds).
    pub exp: usize,
    /// Issued-at timestamp (Unix epoch seconds).
    pub iat: usize,
    /// Unique token ID, used for revocation.
    pub jti: String,
}

/// Manages JWT signing and verification keys.
///
/// Create once at startup. Share via `Arc<JwtManager>`.
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_expiry_seconds: u64,
}

impl JwtManager {
    /// Create a new `JwtManager` from PEM-encoded RSA key pair.
    ///
    /// Returns `AuthError::Jwt` if either key is malformed.
    pub fn new(
        private_key_pem: &str,
        public_key_pem: &str,
        access_expiry_seconds: u64,
    ) -> Result<Self, AuthError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(AuthError::Jwt)?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(AuthError::Jwt)?;
        Ok(Self {
            encoding_key,
            decoding_key,
            access_expiry_seconds,
        })
    }

    /// Issue a new RS256 access token for `(user_id, tenant_id, role)`.
    pub fn generate_access_token(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        role: &str,
    ) -> Result<String, AuthError> {
        let now = Utc::now().timestamp() as usize;
        let exp = now + self.access_expiry_seconds as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            role: role.to_string(),
            exp,
            iat: now,
            jti: Uuid::new_v4().to_string(),
        };
        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key)
            .map_err(AuthError::Jwt)
    }

    /// Validate an RS256 JWT and return its claims.
    ///
    /// Checks signature, expiration, and algorithm. Does NOT check
    /// revocation — the gateway handles that by calling `ValidateToken`,
    /// which can consult a Redis revocation set.
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    AuthError::TokenExpired
                }
                _ => AuthError::TokenInvalid,
            })
    }
}
