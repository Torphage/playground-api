//! Request-boundary authentication abstraction.

use async_trait::async_trait;
use axum::http::request::Parts;

use crate::application::error::AppError;

use super::authenticated_identity::AuthenticatedIdentity;

/// Authenticates an incoming HTTP request.
///
/// Returning:
/// - `Ok(Some(identity))` means authentication succeeded.
/// - `Ok(None)` means no supported credentials were present.
/// - `Err(...)` means credentials were present but invalid, or another
///   authentication failure occurred.
#[async_trait]
pub trait RequestAuthenticator: Send + Sync {
    async fn authenticate(&self, parts: &Parts) -> Result<Option<AuthenticatedIdentity>, AppError>;
}
