//! Composite request authenticator.
//!
//! Tries multiple request authenticators in order and returns the first
//! successful authenticated identity.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::request::Parts;

use crate::api::authentication::AuthenticatedIdentity;
use crate::api::authentication::RequestAuthenticator;
use crate::application::error::AppError;

/// Tries several authentication methods in sequence.
#[derive(Clone, Default)]
pub struct CompositeRequestAuthenticator {
    authenticators: Vec<Arc<dyn RequestAuthenticator>>,
}

impl CompositeRequestAuthenticator {
    /// Creates an empty composite authenticator.
    pub fn new() -> Self {
        Self {
            authenticators: Vec::new(),
        }
    }

    /// Creates a composite authenticator from a pre-built list.
    pub fn with_authenticators(authenticators: Vec<Arc<dyn RequestAuthenticator>>) -> Self {
        Self { authenticators }
    }

    /// Appends an authenticator to the chain.
    ///
    /// Authenticators are tried in insertion order.
    pub fn push(mut self, authenticator: Arc<dyn RequestAuthenticator>) -> Self {
        self.authenticators.push(authenticator);
        self
    }

    /// Returns true if no authenticators have been configured.
    pub fn is_empty(&self) -> bool {
        self.authenticators.is_empty()
    }
}

#[async_trait]
impl RequestAuthenticator for CompositeRequestAuthenticator {
    async fn authenticate(&self, parts: &Parts) -> Result<Option<AuthenticatedIdentity>, AppError> {
        for authenticator in &self.authenticators {
            match authenticator.authenticate(parts).await {
                Ok(Some(identity)) => return Ok(Some(identity)),
                Ok(None) => continue,
                Err(err) => return Err(err),
            }
        }

        Ok(None)
    }
}
