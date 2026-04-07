//! Request-boundary authentication abstraction.
//!
//! This trait is intentionally HTTP-facing: it inspects incoming request parts
//! and either authenticates the caller or reports that no supported
//! credentials were present.

use async_trait::async_trait;
use axum::http::request::Parts;

use crate::application::error::AppError;
use crate::application::platform::authentication::AuthenticatedIdentity;

/// The result of attempting to authenticate an incoming request.
#[derive(Debug, Clone)]
pub enum AuthenticationOutcome {
    /// Authentication succeeded and produced a caller identity.
    Authenticated(AuthenticatedIdentity),

    /// No supported credentials were present in the request.
    ///
    /// This is not an error. It allows higher-level components to decide
    /// whether authentication is optional or required.
    NotPresent,
}

impl AuthenticationOutcome {
    /// Returns the authenticated identity if authentication succeeded.
    pub fn into_identity(self) -> Option<AuthenticatedIdentity> {
        match self {
            Self::Authenticated(identity) => Some(identity),
            Self::NotPresent => None,
        }
    }
}

/// Authenticates an incoming HTTP request.
///
/// Returning:
/// - `Ok(AuthenticationOutcome::Authenticated(identity))` means
///   authentication succeeded.
/// - `Ok(AuthenticationOutcome::NotPresent)` means no supported credentials
///   were present.
/// - `Err(...)` means credentials were present but invalid, or another
///   authentication failure occurred.
#[async_trait]
pub trait RequestAuthenticator: Send + Sync {
    async fn authenticate(&self, parts: &Parts) -> Result<AuthenticationOutcome, AppError>;
}
