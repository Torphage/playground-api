//! Permission value object.
//!
//! Represents a granular access right within the system (e.g., "kitchen.recipe.create").

use serde::{Deserialize, Serialize};
use std::fmt;

/// A strongly-typed permission slug.
///
/// Wraps a string to prevent accidental mixing with other identifiers like
/// Role IDs or User IDs.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(String);

impl Permission {
    /// Constructs a new Permission from a raw slug.
    pub fn new(slug: impl Into<String>) -> Self {
        Self(slug.into())
    }

    /// Returns a reference to the underlying slug string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub const ACCOUNT_SELF_CHANGE_EMAIL: &'static str = "account.self.change.email";

    pub fn identity_self_change_email() -> Self {
        Self::new(Self::ACCOUNT_SELF_CHANGE_EMAIL)
    }
}

impl AsRef<str> for Permission {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Permission({})", self.0)
    }
}
