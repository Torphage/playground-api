//! Authenticated request identity.
//!
//! This is intentionally small. It represents the caller identity established
//! at the request boundary, before loading a richer authorization principal.

use crate::domain::identity::values::UserId;

/// A successfully authenticated caller identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedIdentity {
    pub user_id: UserId,
}

impl AuthenticatedIdentity {
    /// Constructs a new authenticated identity.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }
}
