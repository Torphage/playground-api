use async_trait::async_trait;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::platform::authentication::{
    AuthenticatedIdentity, AuthenticationContext, AuthenticationOutcome, Authenticator,
};
use crate::domain::platform::identity::values::UserId;

use super::JwtVerifier;

/// Authenticates callers using bearer JWTs.
#[derive(Clone)]
pub struct JwtBearerAuthenticator {
    verifier: JwtVerifier,
}

impl JwtBearerAuthenticator {
    pub fn new(verifier: JwtVerifier) -> Self {
        Self { verifier }
    }
}

#[async_trait]
impl Authenticator for JwtBearerAuthenticator {
    async fn authenticate(
        &self,
        context: &AuthenticationContext,
    ) -> Result<AuthenticationOutcome, AppError> {
        let Some(token) = context.bearer_token() else {
            return Ok(AuthenticationOutcome::NotPresent);
        };

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
