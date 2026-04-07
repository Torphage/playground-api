//! Command DTO for revoking the current JWT refresh-token family.

pub struct RevokeTokenCommand {
    pub refresh_token: String,
}
