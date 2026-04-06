//! Command DTO for revoking all refresh tokens for a user.

use crate::domain::platform::::values::UserId;

pub struct RevokeUserTokensCommand {
    pub user_id: UserId,
}
