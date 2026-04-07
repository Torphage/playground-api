//! Access-token issuance contract.

use crate::application::error::AppError;
use crate::domain::platform::identity::values::UserId;

/// A newly issued access token plus client-facing lifetime metadata.
#[derive(Debug, Clone)]
pub struct IssuedAccessToken {
    pub token: String,
    pub expires_in: i64,
}

/// Issues signed access tokens for authenticated users.
pub trait AccessTokenIssuer: Send + Sync {
    fn issue_access_token(&self, user_id: &UserId) -> Result<IssuedAccessToken, AppError>;
}
