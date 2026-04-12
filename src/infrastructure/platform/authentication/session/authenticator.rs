//! Session-backed authenticator.

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::platform::authentication::{
    AuthenticatedIdentity, AuthenticationContext, AuthenticationOutcome, Authenticator,
};
use crate::domain::platform::identity::values::UserId;

use super::fred_store::FredSessionStore;

/// Authenticates callers using a session stored in Redis.
#[derive(Clone)]
pub struct SessionAuthenticator {
    store: Arc<FredSessionStore>,
}

impl SessionAuthenticator {
    /// Creates a new session authenticator.
    pub fn new(store: Arc<FredSessionStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Authenticator for SessionAuthenticator {
    async fn authenticate(
        &self,
        context: &AuthenticationContext,
    ) -> Result<AuthenticationOutcome, AppError> {
        let Some(session_id) = context.session_id() else {
            return Ok(AuthenticationOutcome::NotPresent);
        };

        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| AppError::Authentication("Invalid or expired session".into()))?;

        let user_uuid = Uuid::parse_str(&session.user_id).map_err(|e| {
            AppError::Infrastructure(format!("Stored session user_id is not a valid UUID: {e}"))
        })?;

        let user_id = UserId::from_uuid(user_uuid);

        Ok(AuthenticationOutcome::Authenticated(
            AuthenticatedIdentity::new(user_id),
        ))
    }
}
