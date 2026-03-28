use crate::domain::shared::error::ErrorCode;
use derive_more::{AsRef, Display};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UsernameError {
    #[error("The username contains did not pass validation.")]
    ValidationFailed,
}

impl ErrorCode for UsernameError {
    fn error_code(&self) -> &'static str {
        match self {
            Self::ValidationFailed => "IDENTITY_USERNAME_VALIDATION_FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, AsRef, Serialize, Deserialize)]
#[as_ref(str)]
#[serde(try_from = "String")]
pub struct Username(String);

impl TryFrom<String> for Username {
    type Error = UsernameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let sanitized = trimmed.to_lowercase();

        // TODO: Add some profanity filter here
        if sanitized.len() > 0 {
            // do not return the sanitized string to keep the original capitalization.
            Ok(Self(trimmed.to_string()))
        } else {
            Err(UsernameError::ValidationFailed)
        }
    }
}

impl FromStr for Username {
    type Err = UsernameError;

    /// Allows creating an `Username` directly from a string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl Username {
    /// Returns a reference to the underlying validated string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
