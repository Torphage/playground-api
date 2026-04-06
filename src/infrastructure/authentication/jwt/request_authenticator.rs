use async_trait::async_trait;
use axum::http::{header::AUTHORIZATION, request::Parts};
use uuid::Uuid;

use crate::api::authentication::{AuthenticationOutcome, RequestAuthenticator};
use crate::application::authentication::AuthenticatedIdentity;
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
    async fn authenticate(&self, parts: &Parts) -> Result<AuthenticationOutcome, AppError> {
        let Some(raw_header) = parts.headers.get(AUTHORIZATION) else {
            return Ok(AuthenticationOutcome::NotPresent);
        };

        let header_value = raw_header.to_str().map_err(|_| {
            AppError::Authentication("Authorization header is not valid ASCII".into())
        })?;

        let mut segments = header_value.split_whitespace();

        let Some(scheme) = segments.next() else {
            return Err(AppError::Authentication(
                "Authorization header is empty".into(),
            ));
        };

        let Some(token) = segments.next() else {
            return Err(AppError::Authentication(
                "Bearer token is missing from Authorization header".into(),
            ));
        };

        if segments.next().is_some() {
            return Err(AppError::Authentication(
                "Authorization header must contain exactly two parts".into(),
            ));
        }

        if !scheme.eq_ignore_ascii_case("Bearer") {
            return Err(AppError::Authentication(format!(
                "Unsupported authorization scheme: {scheme}"
            )));
        }

        let claims = self.verifier.verify(token)?;

        let user_uuid = Uuid::parse_str(&claims.sub).map_err(|e| {
            AppError::Authentication(format!("JWT subject is not a valid UUID: {e}"))
        })?;

        let user_id = UserId::from_uuid(user_uuid);

        Ok(AuthenticationOutcome::Authenticated(
            AuthenticatedIdentity::new(user_id),
        ))
    }
}
