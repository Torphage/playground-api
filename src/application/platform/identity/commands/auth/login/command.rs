//! Command DTO for password-based session login.

pub struct LoginCommand {
    pub email: String,
    pub password: String,
}
