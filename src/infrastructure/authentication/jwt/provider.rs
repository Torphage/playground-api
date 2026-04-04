use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

use crate::application::error::AppError;
use crate::application::ports::TokenGenerator;
use crate::config::JwtConfig;
use crate::domain::identity::entities::User;

use super::Claims;

/// A concrete implementation of the `TokenGenerator` using HS256 JWTs.
pub struct JwtProvider {
    encoding_key: EncodingKey,
    expiration_hours: i64,
}

impl JwtProvider {
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            expiration_hours: 24,
        }
    }
}

impl TokenGenerator for JwtProvider {
    fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let expiration = now + Duration::hours(self.expiration_hours);

        let role_slugs: Vec<String> = user.roles.iter().map(|r| r.id.clone()).collect();

        let claims = Claims {
            sub: user.id.as_uuid().to_string(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
            roles: role_slugs,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            tracing::error!("Failed to generate JWT: {}", e);
            AppError::Infrastructure("Failed to sign authentication token".into())
        })?;

        Ok(token)
    }
}
