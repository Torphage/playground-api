//! Application-facing authentication abstraction.

use async_trait::async_trait;

use crate::application::error::AppError;
use crate::application::platform::authentication::{
    authentication_context::AuthenticationContext, authentication_outcome::AuthenticationOutcome,
};

/// Authenticates a caller from transport-neutral authentication context.
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Attempts authentication from the provided context.
    ///
    /// Returning:
    /// - `Ok(AuthenticationOutcome::Authenticated(identity))` means
    ///   authentication succeeded.
    /// - `Ok(AuthenticationOutcome::NotPresent)` means this authenticator found
    ///   no applicable authentication material.
    /// - `Err(...)` means applicable authentication material was present but
    ///   invalid, or another authentication failure occurred.
    async fn authenticate(
        &self,
        context: &AuthenticationContext,
    ) -> Result<AuthenticationOutcome, AppError>;
}
