//! Domain errors specific to the Authentication bounded context.

use thiserror::Error;
use serde_json::Value;
use crate::domain::shared::error::ErrorCode;
use crate::domain::identity::values::email::EmailError;
use crate::domain::identity::values::password::PasswordError;
use crate::domain::identity::values::username::UsernameError;

/// Represents a business rule violation within the Authentication domain.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// The user provided an incorrect email or password.
    #[error("Invalid credentials provided.")]
    InvalidCredentials,

    /// The requested user account could not be found.
    #[error("The specified account does not exist.")]
    AccountNotFound,

    /// The user is attempting to register with an email that is already in use.
    #[error("An account with this email address already exists.")]
    EmailAlreadyExists,

    #[error("Password invalid: {0}")]
    PasswordValidation(#[from] PasswordError),

    #[error(transparent)]
    EmailValidation(#[from] EmailError),

    #[error(transparent)]
    UsernameValidation(#[from] UsernameError),
}

impl ErrorCode for IdentityError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "AUTH_INVALID_CREDENTIALS",
            Self::AccountNotFound => "AUTH_ACCOUNT_NOT_FOUND",
            Self::EmailAlreadyExists => "AUTH_EMAIL_ALREADY_EXISTS",
            // Delegate to the inner granular error trait implementation
            Self::PasswordValidation(err) => err.error_code(),
            Self::EmailValidation(err) => err.error_code(),
            Self::UsernameValidation(err) => err.error_code(),
        }
    }

    fn context(&self) -> Option<Value> {
        match self {
            // Delegate context extraction to the inner granular error
            Self::EmailValidation(err) => err.context(),
            Self::UsernameValidation(err) => err.context(),
            Self::PasswordValidation(err) => err.context(),
            // Other errors currently don't require dynamic context
            _ => None,
        }
    }
}
