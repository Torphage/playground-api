//! Composite request authenticator.
//!
//! Tries multiple request authenticators in order and returns the first
//! successful authenticated identity.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::request::Parts;

use crate::api::authentication::{AuthenticationOutcome, RequestAuthenticator};
use crate::application::error::AppError;

/// Tries several authentication methods in sequence.
///
/// Policy:
/// - The first successful authentication wins.
/// - `NotPresent` falls through to the next authenticator.
/// - Any error stops the chain immediately.
#[derive(Clone, Default)]
pub struct CompositeRequestAuthenticator {
    authenticators: Vec<Arc<dyn RequestAuthenticator>>,
}

impl CompositeRequestAuthenticator {
    /// Creates an empty composite authenticator.
    pub fn new(authenticators: Vec<Arc<dyn RequestAuthenticator>>) -> Self {
        Self { authenticators }
    }

    /// Returns true if no authenticators have been configured.
    pub fn is_empty(&self) -> bool {
        self.authenticators.is_empty()
    }
}

#[async_trait]
impl RequestAuthenticator for CompositeRequestAuthenticator {
    async fn authenticate(&self, parts: &Parts) -> Result<AuthenticationOutcome, AppError> {
        for authenticator in &self.authenticators {
            match authenticator.authenticate(parts).await? {
                AuthenticationOutcome::Authenticated(identity) => {
                    return Ok(AuthenticationOutcome::Authenticated(identity));
                }
                AuthenticationOutcome::NotPresent => continue,
            }
        }

        Ok(AuthenticationOutcome::NotPresent)
    }
}
