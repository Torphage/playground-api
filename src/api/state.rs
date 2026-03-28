//! Shared API state.
//!
//! This module holds the concrete dependencies assembled during application
//! startup and injected into Axum handlers.

use std::sync::Arc;

use crate::config::Config;
use crate::infrastructure::crypto::Argon2Provider;
use crate::infrastructure::db::PostgresTransactionManager;
use crate::infrastructure::repositories::identity::PostgresUserRepository;

/// Repository dependencies used by HTTP handlers.
#[derive(Clone)]
pub struct Repositories {
    /// Concrete PostgreSQL-backed user repository.
    pub user: Arc<PostgresUserRepository>,
}

/// Cryptographic dependencies used by HTTP handlers.
#[derive(Clone)]
pub struct Crypto {
    /// Password hashing provider.
    pub password_hasher: Arc<dyn PasswordHasher>,
}

/// The fully assembled shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Repository implementations.
    pub repos: Repositories,

    /// Transaction manager used by application use cases.
    pub tx_manager: PostgresTransactionManager,

    /// Cryptographic components.
    pub crypto: Crypto,

    /// Read-only application configuration.
    pub config: Arc<Config>,
}
