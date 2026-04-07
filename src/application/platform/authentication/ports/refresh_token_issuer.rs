//! Refresh-token issuance contract.

use crate::application::error::AppError;

/// Issues opaque refresh tokens for one-time return to the client.
pub trait RefreshTokenIssuer: Send + Sync {
    fn issue_refresh_token(&self) -> Result<String, AppError>;
}
