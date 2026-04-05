//! User identity value object.

use uuid::Uuid;

/// Uniquely identifies a user within the system.
///
/// Wraps a UUID to provide type safety and prevent accidental mixing
/// with other UUID-based identifiers (e.g., `SessionId` or `OrderId`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    /// Generates a new, random `UserId` (v4 UUID).
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Reconstructs a `UserId` from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}
