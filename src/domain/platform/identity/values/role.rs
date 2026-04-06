//! Role value object.
//!
//! Represents a collection of permissions grouped under a specific
//! job function or access tier (e.g., "admin", "editor").

use serde::{Deserialize, Serialize};
use std::fmt;

/// A strictly defined user role.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Role {
    /// The programmatic identifier for the role (e.g., "admin").
    pub id: String,

    /// The human-readable display name (e.g., "Administrator").
    pub name: String,
}

impl Role {
    /// Constructs a new Role.
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
        }
    }
}

impl fmt::Debug for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Role({}: {})", self.id, self.name)
    }
}
