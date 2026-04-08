//! Opaque-token issuance contract.

use crate::application::error::AppError;

/// Issues opaque high-entropy tokens for one-time return to clients.
pub trait OpaqueTokenIssuer: Send + Sync {
    fn issue_token(&self) -> Result<String, AppError>;
}
