//! Command DTO for user registration.

/// The intent to register a new user in the system.
///
/// Commands contain raw, unvalidated primitive data. Validation is performed
/// by the application handler.
pub struct RegisterCommand {
    pub username: String,
    pub email: String,
    pub password: String,
}
