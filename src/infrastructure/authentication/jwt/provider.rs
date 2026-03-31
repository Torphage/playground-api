//! JSON Web Token (JWT) infrastructure adapter.

use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};

use crate::application::error::AppError;
use crate::application::ports::TokenGenerator;
use crate::config::AuthConfig;
use crate::domain::identity::entities::User;

// =========================================================================
// JWT CLAIMS DTO
// =========================================================================

/// The data payload embedded inside the JWT.
///
/// These fields follow the RFC 7519 standard for JSON Web Tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (The user's unique identifier)
    pub sub: String,
    /// Issued At (When the token was created)
    pub iat: usize,
    /// Expiration Time (When the token becomes invalid)
    pub exp: usize,
    /// Custom claim: The user's assigned roles
    pub roles: Vec<String>,
}

// =========================================================================
// THE JWT PROVIDER
// =========================================================================

/// A concrete implementation of the `TokenGenerator` using HS256 JWTs.
pub struct JwtProvider {
    encoding_key: EncodingKey,
    /// Token lifespan in hours
    expiration_hours: i64,
}

impl JwtProvider {
    /// Constructs a new JwtProvider using the application's AuthConfig.
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            expiration_hours: 24, // Configurable in a real app
        }
    }
}

impl TokenGenerator for JwtProvider {
    fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.expiration_hours);

        // Map the domain roles into simple string representations
        let role_slugs: Vec<String> = user.roles.iter().map(|r| r.id.clone()).collect();

        let claims = Claims {
            sub: user.id.as_uuid().to_string(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
            roles: role_slugs,
        };

        // Encode and sign the token
        let token = encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            tracing::error!("Failed to generate JWT: {}", e);
            AppError::Infrastructure("Failed to sign authentication token".into())
        })?;

        Ok(token)
    }
}
