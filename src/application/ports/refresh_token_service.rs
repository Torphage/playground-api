//! Application-level contract for opaque refresh-token generation and hashing.

use crate::application::error::AppError;

/// Generates opaque refresh tokens and hashes them for durable storage.
///
/// Refresh tokens are random high-entropy secrets. The raw token is returned to
/// the client once, while only the hash is stored server-side.
pub trait RefreshTokenService: Send + Sync {
    /// Generates a new raw refresh token suitable for returning to the client.
    fn generate_token(&self) -> Result<String, AppError>;

    /// Computes a stable one-way hash for a raw refresh token.
    fn hash_token(&self, token: &str) -> Result<String, AppError>;
}
