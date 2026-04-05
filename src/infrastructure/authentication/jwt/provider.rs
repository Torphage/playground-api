use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::ports::TokenGenerator;
use crate::config::JwtConfig;
use crate::domain::accounts::entities::User;

use super::Claims;

/// A concrete implementation of the `TokenGenerator` using HS256 JWTs.
pub struct JwtProvider {
    encoding_key: EncodingKey,
    issuer: String,
    audience: String,
    access_ttl_seconds: i64,
}

impl JwtProvider {
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
            access_ttl_seconds: config.access_ttl_seconds,
        }
    }
}

impl TokenGenerator for JwtProvider {
    fn generate_token(&self, user: &User) -> Result<String, AppError> {
        let now = Utc::now();
        let expiration = now + Duration::seconds(self.access_ttl_seconds);

        let role_slugs: Vec<String> = user.roles.iter().map(|r| r.id.clone()).collect();

        let claims = Claims {
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            sub: user.id.as_uuid().to_string(),
            iat: now.timestamp() as usize,
            exp: expiration.timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            roles: role_slugs,
        };

        let token = encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            tracing::error!("Failed to generate JWT: {}", e);
            AppError::Infrastructure("Failed to sign authentication token".into())
        })?;

        Ok(token)
    }
}