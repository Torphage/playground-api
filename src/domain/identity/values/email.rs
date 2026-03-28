//! Email address value object and associated validation logic.
//!
//! This module defines the `Email` aggregate, ensuring that invalid email
//! addresses cannot be instantiated in memory. It uses external parsing
//! libraries for strict RFC compliance and masks Personally Identifiable
//! Information (PII) during logging.

use derive_more::{AsRef, Display};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

use crate::domain::shared::ErrorCode;

// =========================================================================
// ERRORS
// =========================================================================

/// Specific validation failures that can occur when parsing an email.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum EmailError {
    #[error("The email format is invalid.")]
    InvalidFormat,

    #[error("The email belongs to a blocked or disposable domain.")]
    BlockedDomain,
}

impl ErrorCode for EmailError {
    /// Maps validation failures to localized frontend slugs.
    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidFormat => "IDENTITY_EMAIL_INVALID_FORMAT",
            Self::BlockedDomain => "IDENTITY_EMAIL_BLOCKED_DOMAIN",
        }
    }
}

// =========================================================================
// VALUE OBJECT
// =========================================================================

/// A strongly-typed, validated email address.
///
/// This struct guarantees that the underlying string is a valid email address.
/// It integrates with `serde` to automatically reject invalid payloads at the
/// API boundary.
#[derive(Clone, PartialEq, Eq, Hash, Display, AsRef, Serialize, Deserialize)]
#[as_ref(str)]
#[serde(try_from = "String")]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = EmailError;

    /// The single source of truth for constructing an `Email`.
    ///
    /// Trims whitespace, normalizes to lowercase, and validates against
    /// standard email formatting rules.
    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let sanitized = raw.trim().to_lowercase();

        // Delegate to a robust, RFC-compliant parser
        if email_address::EmailAddress::is_valid(&sanitized) {
            // Placeholder for business logic: rejecting disposable domains
            if sanitized.ends_with("@tempmail.com") {
                // TODO
                return Err(EmailError::BlockedDomain);
            }
            Ok(Self(sanitized))
        } else {
            Err(EmailError::InvalidFormat)
        }
    }
}

impl FromStr for Email {
    type Err = EmailError;

    /// Allows creating an `Email` directly from a string slice.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl Email {
    /// Returns a reference to the underlying validated string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// =========================================================================
// SECURITY & LOGGING
// =========================================================================

impl fmt::Debug for Email {
    /// Custom debug formatter that masks PII.
    ///
    /// Prevents sensitive user data from leaking into application logs while
    /// preserving enough information (the domain) for debugging purposes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let email = &self.0;
        if let Some((local, domain)) = email.split_once('@') {
            let masked = if local.len() > 2 {
                format!("{}***@{}", &local[..2], domain)
            } else {
                format!("***@{}", domain)
            };
            write!(f, "EmailAddress({})", masked)
        } else {
            // Fallback, though an invalid email shouldn't exist in this struct
            write!(f, "EmailAddress(***)")
        }
    }
}
