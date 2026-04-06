//! Password value objects and complexity validation.
//!
//! This module defines the strict type boundaries for password handling. It
//! separates unhashed input (`PlaintextPassword`) from secure storage
//! (`PasswordHash`) to eliminate the risk of storing cleartext passwords.

use serde::Deserialize;
use serde_json::{Value, json};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::shared::ErrorCode;

// =========================================================================
// ERRORS
// =========================================================================

/// Minimum required password length according to the current security policy.
pub const MIN_PASSWORD_LENGTH: usize = 8;
/// Maximum allowed password length according to the current security policy.
pub const MAX_PASSWORD_LENGTH: usize = 128;

/// Specific validation failures that occur when a password violates security policies.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PasswordError {
    #[error("Password cannot be empty")]
    Empty,
    #[error("Password must be at least {0} characters")]
    TooShort(usize),
    #[error("Password exceeds the maximum allowed length of 128 characters")]
    TooLong(usize),
    #[error("Password must contain at least one lowercase letter")]
    MissingLowercase,
    #[error("Password must contain at least one uppercase letter")]
    MissingUppercase,
    #[error("Password must contain at least one number")]
    MissingNumber,
    #[error("Password must contain at least one symbol")]
    MissingSymbol,
}

impl ErrorCode for PasswordError {
    /// Maps validation failures to localized frontend slugs.
    fn error_code(&self) -> &'static str {
        match self {
            Self::Empty => "IDENTITY_PASSWORD_EMPTY",
            Self::TooShort(_) => "IDENTITY_PASSWORD_TOO_SHORT",
            Self::TooLong(_) => "IDENTITY_PASSWORD_TOO_LONG",
            Self::MissingLowercase => "IDENTITY_PASSWORD_MISSING_LOWERCASE",
            Self::MissingUppercase => "IDENTITY_PASSWORD_MISSING_UPPERCASE",
            Self::MissingNumber => "IDENTITY_PASSWORD_MISSING_NUMBER",
            Self::MissingSymbol => "IDENTITY_PASSWORD_MISSING_SYMBOL",
        }
    }

    /// Provides dynamic context for frontend translations.
    fn context(&self) -> Option<Value> {
        match self {
            Self::TooShort(min) => Some(json!({ "min_length": min })),
            _ => None,
        }
    }
}

// =========================================================================
// PLAINTEXT PASSWORD (INPUT)
// =========================================================================

/// A validated, unhashed password.
///
/// This type guarantees that the underlying string meets the application's
/// complexity and length requirements. It is exclusively used during
/// registration and login flows and must be hashed before storage.
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct PlaintextPassword(String);

impl TryFrom<String> for PlaintextPassword {
    type Error = PasswordError;

    /// Validates raw input against the system's password security policy.
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(PasswordError::Empty);
        }

        if raw.len() < MIN_PASSWORD_LENGTH {
            return Err(PasswordError::TooShort(MIN_PASSWORD_LENGTH));
        }

        if raw.len() > MAX_PASSWORD_LENGTH {
            return Err(PasswordError::TooLong(MAX_PASSWORD_LENGTH)); // Prevent DoS during hashing
        }

        if !raw.chars().any(|c| c.is_numeric()) {
            return Err(PasswordError::MissingNumber);
        }

        if !raw.chars().any(|c| c.is_lowercase()) {
            return Err(PasswordError::MissingLowercase);
        }

        if !raw.chars().any(|c| c.is_uppercase()) {
            return Err(PasswordError::MissingUppercase);
        }

        // Checks for standard ASCII punctuation (e.g., !@#$%)
        if !raw.chars().any(|c| c.is_ascii_punctuation()) {
            return Err(PasswordError::MissingSymbol);
        }

        Ok(Self(raw))
    }
}

impl FromStr for PlaintextPassword {
    type Err = PasswordError;

    /// Allows creating a `PlaintextPassword` directly from a string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl PlaintextPassword {
    /// Returns a reference to the validated plaintext string.
    ///
    /// This should only be called by the `PasswordHasher` port within the
    /// application layer.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the wrapper to give back the string
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Debug for PlaintextPassword {
    /// Custom debug formatter that masks the password entirely.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlaintextPassword(***REDACTED***)")
    }
}

// =========================================================================
// PASSWORD HASH (STORAGE)
// =========================================================================

/// A securely hashed password ready for database storage.
///
/// This type can only be created by the cryptography infrastructure adapter.
#[derive(Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    pub fn new(hash: String) -> Self {
        Self(hash)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PasswordHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PasswordHash(***REDACTED***)")
    }
}

// =========================================================================
// TESTS
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_password_fails() {
        let result = PlaintextPassword::try_from("".to_string());
        assert!(matches!(result, Err(PasswordError::Empty)));
    }

    #[test]
    fn test_short_password_fails() {
        let result = PlaintextPassword::try_from("Ab1$".to_string());
        assert!(matches!(result, Err(PasswordError::TooShort(_))));
    }

    #[test]
    fn test_no_lowercase_letter_in_password_fails() {
        let result = PlaintextPassword::try_from("UPPERCASE_1$".to_string());
        assert!(matches!(result, Err(PasswordError::MissingLowercase)));
    }

    #[test]
    fn test_no_uppercase_letter_in_password_fails() {
        let result = PlaintextPassword::try_from("lowercase_1$".to_string());
        assert!(matches!(result, Err(PasswordError::MissingUppercase)));
    }

    #[test]
    fn test_no_number_in_password_fails() {
        let result = PlaintextPassword::try_from("NoNumber_$".to_string());
        assert!(matches!(result, Err(PasswordError::MissingNumber)));
    }

    #[test]
    fn test_no_symbol_in_password_fails() {
        let result = PlaintextPassword::try_from("NoSymbol88".to_string());
        assert!(matches!(result, Err(PasswordError::MissingSymbol)));
    }

    #[test]
    fn test_valid_password_works() {
        let result = "Password123!".parse::<PlaintextPassword>();
        assert!(result.is_ok());
    }
}
