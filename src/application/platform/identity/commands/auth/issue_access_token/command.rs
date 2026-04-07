//! Command DTO for password-based JWT issuance.

pub struct IssueTokenCommand {
    pub email: String,
    pub password: String,
}
