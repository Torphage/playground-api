//! Command DTO for revoking all refresh tokens for a user.

use crate::domain::platform::identity::values::UserId;

pub struct RevokeUserTokensCommand {
    pub user_id: UserId,
}
