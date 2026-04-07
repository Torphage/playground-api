use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::{AccessTokenIssuer, IssuedAccessToken};
use crate::config::JwtConfig;
use crate::domain::platform::identity::values::UserId;

use super::Claims;

/// A concrete implementation of the `JwtAccessTokenIssuer` using HS256 JWTs.
pub struct JwtAccessTokenIssuer {
    encoding_key: EncodingKey,
    issuer: String,
    audience: String,
    access_ttl_seconds: i64,
}

impl JwtAccessTokenIssuer {
    pub fn new(config: &JwtConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(config.secret.as_bytes()),
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
            access_ttl_seconds: config.access_ttl_seconds,
        }
    }
}

impl AccessTokenIssuer for JwtAccessTokenIssuer {
    fn issue_access_token(&self, user_id: &UserId) -> Result<IssuedAccessToken, AppError> {
        let now = Utc::now();
        let expiration = now + Duration::seconds(self.access_ttl_seconds);

        let claims = Claims {
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            sub: user_id.as_uuid().to_string(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        let header = Header::new(Algorithm::HS256);

        let token = encode(&header, &claims, &self.encoding_key).map_err(|e| {
            tracing::error!("Failed to generate JWT: {}", e);
            AppError::Infrastructure("Failed to sign authentication token".into())
        })?;

        Ok(IssuedAccessToken {
            token,
            expires_in: self.access_ttl_seconds,
        })
    }
}
