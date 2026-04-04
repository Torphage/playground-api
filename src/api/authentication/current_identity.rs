//! Axum extractor for an authenticated identity.

use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;

use super::authenticated_identity::AuthenticatedIdentity;
use super::request_authenticator::RequestAuthenticator;
use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::error::AppError;

/// Authentication-specific state extracted from the full app state.
#[derive(Clone)]
pub struct AuthenticationState {
    pub request_authenticator: Arc<dyn RequestAuthenticator>,
}

impl FromRef<AppState> for AuthenticationState {
    fn from_ref(state: &AppState) -> Self {
        Self {
            request_authenticator: state.authentication.request_authenticator.clone(),
        }
    }
}

/// Extracted authenticated identity for the current request.
#[derive(Debug, Clone)]
pub struct CurrentIdentity(pub AuthenticatedIdentity);

impl<S> FromRequestParts<S> for CurrentIdentity
where
    AuthenticationState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_state = AuthenticationState::from_ref(state);

        let identity = auth_state
            .request_authenticator
            .authenticate(parts)
            .await?
            .ok_or_else(|| {
                ApiError(AppError::Authentication(
                    "Missing or unsupported authentication credentials".into(),
                ))
            })?;

        Ok(Self(identity))
    }
}
