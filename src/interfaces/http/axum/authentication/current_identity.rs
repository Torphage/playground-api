//! Axum extractor for the authenticated identity of the current request.

use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use crate::application::error::AppError;
use crate::application::platform::authentication::{
    AuthenticatedIdentity, AuthenticationOutcome, Authenticator,
};
use crate::interfaces::http::axum::{error::ApiError, state::AppState};
use crate::interfaces::http::shared::authentication::authentication_context_from_request_parts;

/// Authentication-specific state extracted from the full app state.
#[derive(Clone)]
struct AuthenticationState {
    authenticator: Arc<dyn Authenticator>,
    session_cookie_name: String,
}

impl FromRef<AppState> for AuthenticationState {
    fn from_ref(state: &AppState) -> Self {
        Self {
            authenticator: state.platform.authentication.authenticator.clone(),
            session_cookie_name: state.platform.authentication.session_cookie_name.clone(),
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

        let context =
            authentication_context_from_request_parts(parts, &auth_state.session_cookie_name)?;

        let identity = match auth_state.authenticator.authenticate(&context).await? {
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
