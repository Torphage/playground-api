//! Bcrypt-based password hasher.

use async_trait::async_trait;
use bcrypt::{DEFAULT_COST, non_truncating_hash, non_truncating_verify};

use crate::application::error::AppError;
use crate::domain::platform::identity::values::{PasswordHash, PlaintextPassword};
use crate::domain::platform::identity::{AccountError, PasswordHasher};

/// Bcrypt-based password hashing implementation.
#[derive(Clone, Debug)]
pub struct BcryptPasswordHasher {
    cost: u32,
}

impl Default for BcryptPasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl BcryptPasswordHasher {
    /// Creates a bcrypt password hasher using the crate default cost.
    pub fn new() -> Self {
        Self { cost: DEFAULT_COST }
    }

    /// Creates a bcrypt password hasher with an explicit cost.
    pub fn with_cost(cost: u32) -> Self {
        Self { cost }
    }
}

#[async_trait]
impl PasswordHasher for BcryptPasswordHasher {
    async fn hash(&self, password: &PlaintextPassword) -> Result<PasswordHash, AccountError> {
        let hash = non_truncating_hash(password.as_str(), self.cost)
            .map_err(|e| AccountError::InvalidCredentials)?;

        Ok(PasswordHash::new(hash))
    }

    async fn verify(
        &self,
        password: &PlaintextPassword,
        hash: &PasswordHash,
    ) -> Result<bool, AccountError> {
        non_truncating_verify(password.as_str(), hash.as_str())
            .map_err(|e| AccountError::InvalidCredentials)
    }
}
