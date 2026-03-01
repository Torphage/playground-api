//! Cryptographic infrastructure using the Argon2id algorithm.
//!
//! This module provides the concrete implementation of the domain's
//! `PasswordHasher` port. It uses the `argon2` crate to securely hash
//! and verify passwords, offloading the CPU-intensive work to a blocking
//! thread pool to prevent async runtime starvation.

use async_trait::async_trait;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash as Argon2Hash,
        PasswordHasher as Argon2HasherTrait,
        PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use crate::domain::identity::error::IdentityError;
use crate::domain::identity::ports::PasswordHasher;
use crate::domain::identity::values::password::{PasswordHash, PlaintextPassword};

/// A concrete password hasher utilizing the Argon2id algorithm.
///
/// Argon2 is the current industry standard and winner of the Password Hashing
/// Competition. This struct requires no state and can be cheaply cloned or
/// shared across multiple threads.
#[derive(Clone, Default)]
pub struct Argon2Provider;

impl Argon2Provider {
    /// Creates a new instance of the Argon2 hasher using default parameters.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PasswordHasher for Argon2Provider {
    /// Hashes a plaintext password securely using a randomly generated salt.
    async fn hash(&self, password: &PlaintextPassword) -> Result<PasswordHash, IdentityError> {
        // We must extract the string to move it across the thread boundary
        let pass_str = password.as_str().to_owned();

        // Offload the CPU-bound hashing process to Tokio's blocking thread pool
        let hash_result = tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();

            argon2
                .hash_password(pass_str.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        })
            .await
            .map_err(|_| {
                tracing::error!("Tokio blocking task panicked during password hashing");
                // If the thread panics, we fail secure and deny the operation
                IdentityError::InvalidCredentials
            })?
            .map_err(|e| {
                tracing::error!("Argon2 hashing failed internally: {}", e);
                IdentityError::InvalidCredentials
            })?;

        Ok(PasswordHash::new(hash_result))
    }

    /// Verifies a plaintext password against an existing secure hash.
    async fn verify(&self, password: &PlaintextPassword, hash: &PasswordHash) -> Result<bool, IdentityError> {
        let pass_str = password.as_str().to_owned();
        let hash_str = hash.as_str().to_owned();

        let is_valid = tokio::task::spawn_blocking(move || {
            // Parse the stored string back into an Argon2 hash object
            let parsed_hash = match Argon2Hash::new(&hash_str) {
                Ok(h) => h,
                Err(e) => {
                    tracing::error!("Stored password hash is malformed: {}", e);
                    return false;
                }
            };

            // Perform the constant-time cryptographic comparison
            Argon2::default()
                .verify_password(pass_str.as_bytes(), &parsed_hash)
                .is_ok()
        })
            .await
            .map_err(|_| {
                tracing::error!("Tokio blocking task panicked during password verification");
                IdentityError::InvalidCredentials
            })?;

        // We return Ok(is_valid) rather than an error on a mismatch,
        // because a wrong password is an expected business outcome, not a system failure.
        Ok(is_valid)
    }
}
