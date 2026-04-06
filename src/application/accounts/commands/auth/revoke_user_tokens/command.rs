//! Command DTO for revoking all refresh tokens for a user.

use crate::domain::accounts::values::UserId;

pub struct RevokeUserTokensCommand {
    pub user_id: UserId,
}
