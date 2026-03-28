//! Cryptographic contracts for password management.
//!
//! This module defines the interface for hashing and verifying passwords,
//! isolating the domain logic from specific cryptographic algorithms.

use crate::domain::identity::error::IdentityError;
use crate::domain::identity::values::password::{PasswordHash, PlaintextPassword};
use async_trait::async_trait;

/// The business interface for securing user credentials.
#[async_trait]
pub trait PasswordHasher: Send + Sync {
    /// Converts a validated plaintext password into a secure, salt-embedded hash.
    ///
    /// This process is typically CPU-intensive and should yield to the async runtime.
    async fn hash(&self, password: &PlaintextPassword) -> Result<PasswordHash, IdentityError>;

    /// Verifies an attempted plaintext password against a stored hash.
    ///
    /// Returns a boolean indicating whether the credentials matched. A mismatched
    /// password does not return an error, but rather `Ok(false)`, as failed
    /// verification is an expected business outcome.
    async fn verify(
        &self,
        password: &PlaintextPassword,
        hash: &PasswordHash,
    ) -> Result<bool, IdentityError>;
}
