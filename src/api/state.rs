//! Shared API state.
//!
//! This module holds the concrete dependencies assembled during application
//! startup and injected into Axum handlers.

use std::sync::Arc;

use axum::extract::FromRef;

use crate::api::authentication::RequestAuthenticator;
use crate::application::authorization::Authorizer;
use crate::application::ports::TokenGenerator;
use crate::config::AppConfig;
use crate::infrastructure::authentication::session::FredSessionStore;
use crate::infrastructure::crypto::Argon2Provider;
use crate::infrastructure::db::postgres::PostgresTransactionManager;
use crate::infrastructure::repositories::identity::{
    PostgresPrincipalLoader, PostgresUserRepository,
};

/// Repository dependencies used by HTTP handlers and application workflows.
#[derive(Clone)]
pub struct Repositories {
    /// Concrete PostgreSQL-backed principal loader.
    pub principal: Arc<PostgresPrincipalLoader>,

    /// Concrete PostgreSQL-backed user repository.
    pub user: Arc<PostgresUserRepository>,
}

/// Cryptographic dependencies used by HTTP handlers.
#[derive(Clone)]
pub struct Crypto {
    /// Password hashing provider.
    pub password_hasher: Arc<Argon2Provider>,
}

/// Incoming-request authentication dependencies.
#[derive(Clone)]
pub struct Authentication {
    /// Mechanism-neutral request authenticator.
    pub request_authenticator: Arc<dyn RequestAuthenticator>,
}

/// Session infrastructure exposed to handlers/use cases.
///
/// Kept separate from `Authentication` because session storage is useful beyond
/// request extraction itself (login, logout, session revocation, etc.).
#[derive(Clone)]
pub struct Sessions {
    /// Redis-backed session store.
    pub store: Arc<FredSessionStore>,
}

/// Outgoing token issuance dependencies.
///
/// Kept separate from `Authentication` because not every authentication method
/// implies token issuance.
#[derive(Clone)]
pub struct TokenIssuance {
    /// Current token generation capability.
    pub token_generator: Arc<dyn TokenGenerator>,
}

/// Authorization dependencies used inside application workflows.
#[derive(Clone)]
pub struct Authorization {
    /// Permission-based authorizer.
    pub authorizer: Arc<dyn Authorizer>,
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

    /// Request-boundary authentication components.
    pub authentication: Authentication,

    /// Session storage infrastructure.
    pub sessions: Sessions,

    /// Token issuance components.
    pub token_issuance: TokenIssuance,

    /// Authorization components.
    pub authorization: Authorization,

    /// Read-only application configuration.
    pub config: Arc<AppConfig>,
}

impl FromRef<AppState> for Authentication {
    fn from_ref(state: &AppState) -> Self {
        state.authentication.clone()
    }
}

impl FromRef<AppState> for Sessions {
    fn from_ref(state: &AppState) -> Self {
        state.sessions.clone()
    }
}

impl FromRef<AppState> for Authorization {
    fn from_ref(state: &AppState) -> Self {
        state.authorization.clone()
    }
}
