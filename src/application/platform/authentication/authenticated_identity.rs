//! Authenticated caller identity.
//!
//! This type represents the actor established by authentication, independent
//! of HTTP, Axum, cookies, JWTs, or any other delivery mechanism.

use crate::domain::platform::::values::UserId;

/// A successfully authenticated caller identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedIdentity {
    user_id: UserId,
}

impl AuthenticatedIdentity {
    /// Constructs a new authenticated identity.
    pub fn new(user_id: UserId) -> Self {
        Self { user_id }
    }

    /// Returns the authenticated user's identifier.
    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    /// Consumes the identity and returns the owned user identifier.
    pub fn into_user_id(self) -> UserId {
        self.user_id
    }
}
