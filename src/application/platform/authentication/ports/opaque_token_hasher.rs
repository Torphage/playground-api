//! Opaque-token hashing contract.

use crate::application::error::AppError;

/// Hashes opaque tokens for durable server-side storage and lookup.
pub trait OpaqueTokenHasher: Send + Sync {
    fn hash_token(&self, token: &str) -> Result<String, AppError>;
}
