use async_trait::async_trait;
use axum::http::{header::AUTHORIZATION, request::Parts};
use uuid::Uuid;

use crate::api::authentication::AuthenticatedIdentity;
use crate::api::authentication::RequestAuthenticator;
use crate::application::error::AppError;
use crate::domain::accounts::values::UserId;

use super::JwtVerifier;

/// Authenticates requests using Bearer JWTs.
#[derive(Clone)]
pub struct JwtRequestAuthenticator {
    verifier: JwtVerifier,
}

impl JwtRequestAuthenticator {
    pub fn new(verifier: JwtVerifier) -> Self {
        Self { verifier }
    }
}

#[async_trait]
impl RequestAuthenticator for JwtRequestAuthenticator {
    async fn authenticate(&self, parts: &Parts) -> Result<Option<AuthenticatedIdentity>, AppError> {
        let Some(header_value) = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
        else {
            return Ok(None);
        };

        let Some(token) = header_value.strip_prefix("Bearer ") else {
            return Ok(None);
        };

        let claims = self.verifier.verify(token)?;

        let user_uuid = Uuid::parse_str(&claims.sub).map_err(|e| {
            AppError::Authentication(format!("JWT subject is not a valid UUID: {e}"))
        })?;

        let user_id = UserId::from_uuid(user_uuid);

        Ok(Some(AuthenticatedIdentity::new(user_id)))
    }
}
