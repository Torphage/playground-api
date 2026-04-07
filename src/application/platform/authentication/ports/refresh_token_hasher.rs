//! Refresh-token hashing contract.

use crate::application::error::AppError;

/// Hashes opaque refresh tokens for durable server-side storage.
pub trait RefreshTokenHasher: Send + Sync {
    fn hash_refresh_token(&self, token: &str) -> Result<String, AppError>;
}
