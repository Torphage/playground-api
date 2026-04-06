//! Shared API state.
//!
//! This module exposes HTTP-facing application capabilities that are prewired
//! during startup. Endpoint files should depend on these capabilities rather
//! than assembling workflows from low-level infrastructure components.

use std::sync::Arc;

use axum::extract::FromRef;

use crate::api::authentication::RequestAuthenticator;
use crate::application::accounts::commands::{
    auth::{issue_access_token, revoke_refresh_token, rotate_refresh_token},
    me::change_my_password,
};
use crate::application::ports::{RefreshTokenService, TokenGenerator};
use crate::config::AppConfig;
use crate::infrastructure::authentication::session::FredSessionStore;
use crate::infrastructure::db::postgres::PostgresTransactionManager;
use crate::infrastructure::repositories::accounts::principals::CacheBackedPrincipalLoader;
use crate::infrastructure::repositories::accounts::{
    PostgresPrincipalLoader, PostgresRefreshTokenRepository, PostgresUserRepository,
};

/// Concrete application handler type for the "change my password" workflow.
pub type ChangeMyPasswordHandler = change_my_password::Handler<
    PostgresTransactionManager,
    PostgresUserRepository,
    CacheBackedPrincipalLoader<PostgresPrincipalLoader>,
>;

/// Concrete application handler type for password-based access-token issuance.
pub type IssueAccessTokenHandler = issue_access_token::IssueTokenHandler<
    PostgresTransactionManager,
    PostgresUserRepository,
    PostgresRefreshTokenRepository,
>;

/// Concrete application handler type for refresh-token rotation.
pub type RotateRefreshTokenHandler = rotate_refresh_token::RefreshTokenHandler<
    PostgresTransactionManager,
    PostgresUserRepository,
    PostgresRefreshTokenRepository,
>;

/// Concrete application handler type for refresh-token revocation.
pub type RevokeRefreshTokenHandler = revoke_refresh_token::RevokeTokenHandler<
    PostgresTransactionManager,
    PostgresRefreshTokenRepository,
>;

/// Incoming-request authentication dependencies.
#[derive(Clone)]
pub struct Authentication {
    /// Mechanism-neutral request authenticator.
    pub request_authenticator: Arc<dyn RequestAuthenticator>,
}

/// Session infrastructure exposed to handlers/use cases.
///
/// Kept separate from request authentication because session storage is useful
/// beyond request extraction itself (login, logout, session revocation, etc.).
#[derive(Clone)]
pub struct Sessions {
    /// Redis-backed session store.
    pub store: Arc<FredSessionStore>,
}

/// Outgoing token issuance dependencies.
///
/// Kept separate from request authentication because not every authentication
/// method implies token issuance.
#[derive(Clone)]
pub struct TokenIssuance {
    /// Current token generation capability.
    pub token_generator: Arc<dyn TokenGenerator>,

    /// Refresh-token generation capability.
    pub refresh_token_service: Arc<dyn RefreshTokenService>,
}

/// Platform-wide capabilities shared across application modules.
///
/// These are cross-cutting concerns that support the whole system rather than
/// belonging to one specific app/module like `accounts`, `kitchen`, or
/// `workout`.
#[derive(Clone)]
pub struct PlatformState {
    /// Request-boundary authentication components.
    pub authentication: Authentication,

    /// Session storage infrastructure.
    pub sessions: Sessions,

    /// Token issuance components.
    pub token_issuance: TokenIssuance,
}

/// Accounts/auth workflows exposed to the API layer.
#[derive(Clone)]
pub struct AccountsAuthHandlers {
    /// Issues an access token and refresh token using credentials.
    pub issue_token: Arc<IssueAccessTokenHandler>,

    /// Rotates a refresh token into a new token pair.
    pub refresh_token: Arc<RotateRefreshTokenHandler>,

    /// Revokes a refresh token.
    pub revoke_token: Arc<RevokeRefreshTokenHandler>,
}

/// Accounts/"me" workflows exposed to the API layer.
#[derive(Clone)]
pub struct AccountsMeHandlers {
    /// Changes the authenticated user's password.
    pub change_my_password: Arc<ChangeMyPasswordHandler>,
}

/// Accounts workflows exposed to the API layer.
#[derive(Clone)]
pub struct AccountsHandlers {
    /// Authentication/token workflows.
    pub auth: AccountsAuthHandlers,

    /// Authenticated self-service account workflows.
    pub me: AccountsMeHandlers,
}

/// Application-module state exposed to the API layer.
///
/// As the super-app grows, additional modules such as `kitchen` and `workout`
/// should be added here as siblings to `accounts`.
#[derive(Clone)]
pub struct AppsState {
    /// Account-related workflows.
    pub accounts: AccountsHandlers,
    // Future examples:
    // pub kitchen: KitchenHandlers,
    // pub workout: WorkoutHandlers,
}

/// The fully assembled shared application state.
///
/// The API layer should primarily depend on:
/// - platform capabilities for cross-cutting concerns
/// - prewired app/module handlers for business workflows
///
/// It should not assemble workflows from raw repositories and low-level
/// infrastructure parts.
#[derive(Clone)]
pub struct AppState {
    /// Cross-cutting platform capabilities.
    pub platform: PlatformState,

    /// Prewired app/module workflows.
    pub apps: AppsState,

    /// Read-only application configuration.
    pub config: Arc<AppConfig>,
}

impl FromRef<AppState> for Authentication {
    fn from_ref(state: &AppState) -> Self {
        state.platform.authentication.clone()
    }
}

impl FromRef<AppState> for Sessions {
    fn from_ref(state: &AppState) -> Self {
        state.platform.sessions.clone()
    }
}

impl FromRef<AppState> for TokenIssuance {
    fn from_ref(state: &AppState) -> Self {
        state.platform.token_issuance.clone()
    }
}

impl FromRef<AppState> for AccountsHandlers {
    fn from_ref(state: &AppState) -> Self {
        state.apps.accounts.clone()
    }
}
