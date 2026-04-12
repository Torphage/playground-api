//! Composite authenticator.
//!
//! Tries multiple authenticators in order and returns the first successful
//! authenticated identity.

use std::sync::Arc;

use async_trait::async_trait;

use crate::application::error::AppError;
use crate::application::platform::authentication::{
    AuthenticationContext, AuthenticationOutcome, Authenticator,
};

/// Tries several authentication methods in sequence.
///
/// Policy:
/// - The first successful authentication wins.
/// - `NotPresent` falls through to the next authenticator.
/// - Any error stops the chain immediately.
#[derive(Clone, Default)]
pub struct CompositeAuthenticator {
    authenticators: Vec<Arc<dyn Authenticator>>,
}

impl CompositeAuthenticator {
    /// Creates a composite authenticator.
    pub fn new(authenticators: Vec<Arc<dyn Authenticator>>) -> Self {
        Self { authenticators }
    }

    /// Returns true if no authenticators have been configured.
    pub fn is_empty(&self) -> bool {
        self.authenticators.is_empty()
    }
}

#[async_trait]
impl Authenticator for CompositeAuthenticator {
    async fn authenticate(
        &self,
        context: &AuthenticationContext,
    ) -> Result<AuthenticationOutcome, AppError> {
        for authenticator in &self.authenticators {
            match authenticator.authenticate(context).await? {
                AuthenticationOutcome::Authenticated(identity) => {
                    return Ok(AuthenticationOutcome::Authenticated(identity));
                }
                AuthenticationOutcome::NotPresent => continue,
            }
        }

        Ok(AuthenticationOutcome::NotPresent)
    }
}
