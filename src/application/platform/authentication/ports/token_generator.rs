//! Access-token issuance contract.

use crate::application::error::AppError;
use crate::domain::platform::::values::UserId;

/// A newly issued access token plus its client-facing lifetime metadata.
#[derive(Debug, Clone)]
pub struct IssuedAccessToken {
    pub token: String,
    pub expires_in: i64,
}

/// The ability to generate a signed access token for an authenticated user.
///
/// This contract is intentionally narrow: access-token generation only depends
/// on the authenticated subject identity, not on a full domain aggregate.
pub trait TokenGenerator: Send + Sync {
    /// Generates a signed access token for the supplied subject.
    fn generate_token(&self, user_id: &UserId) -> Result<IssuedAccessToken, AppError>;
}
