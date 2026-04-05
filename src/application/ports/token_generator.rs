//! Session and token generation contracts.
//!
//! Defines the interface for issuing access tokens to authenticated users.

use crate::application::error::AppError;
use crate::domain::accounts::entities::User;

// =========================================================================
// TOKEN GENERATION PORT
// =========================================================================

/// The ability to generate a secure access token for a given user.
///
/// By defining this as a trait, we isolate the business logic from the
/// specific token standard (e.g., JWT, Paseto, or opaque database tokens).
pub trait TokenGenerator: Send + Sync {
    /// Generates a signed access token containing the user's identity and claims.
    fn generate_token(&self, user: &User) -> Result<String, AppError>;
}
