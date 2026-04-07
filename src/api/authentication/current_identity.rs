//! Axum extractor for the authenticated identity of the current request.

use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::error::AppError;
use crate::application::platform::authentication::AuthenticatedIdentity;

use crate::api::authentication::{AuthenticationOutcome, RequestAuthenticator};

/// Authentication-specific state extracted from the full app state.
#[derive(Clone)]
struct AuthenticationState {
    request_authenticator: Arc<dyn RequestAuthenticator>,
}

impl FromRef<AppState> for AuthenticationState {
    fn from_ref(state: &AppState) -> Self {
        Self {
            request_authenticator: state.platform.authentication.request_authenticator.clone(),
        }
    }
}

/// Extracted authenticated identity for the current request.
#[derive(Debug, Clone)]
pub struct CurrentIdentity(AuthenticatedIdentity);

impl CurrentIdentity {
    /// Returns the authenticated identity.
    pub fn identity(&self) -> &AuthenticatedIdentity {
        &self.0
    }

    /// Consumes the extractor and returns the authenticated identity.
    pub fn into_inner(self) -> AuthenticatedIdentity {
        self.0
    }
}

impl<S> FromRequestParts<S> for CurrentIdentity
where
    AuthenticationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_state = AuthenticationState::from_ref(state);

        let identity = match auth_state.request_authenticator.authenticate(parts).await? {
            AuthenticationOutcome::Authenticated(identity) => identity,
            AuthenticationOutcome::NotPresent => {
                return Err(ApiError(AppError::Authentication(
                    "Missing or unsupported authentication credentials".into(),
                )));
            }
        };

        Ok(Self(identity))
    }
}
