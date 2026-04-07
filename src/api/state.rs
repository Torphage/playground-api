//! Shared API state.
//!
//! This module exposes HTTP-facing application capabilities that are prewired
//! during startup. Endpoint files should depend on these capabilities rather
//! than assembling workflows from low-level infrastructure components.

use std::sync::Arc;

use axum::extract::FromRef;

use crate::api::authentication::RequestAuthenticator;
use crate::application::platform::identity::commands::auth::logout::LogoutHandler;
use crate::application::platform::identity::commands::auth::{login, register_user};
use crate::application::platform::identity::commands::{
    auth::{issue_access_token, revoke_refresh_token, rotate_refresh_token},
    me::change_my_password,
};
use crate::config::AppConfig;
use crate::infrastructure::db::postgres::PostgresTransactionManager;
use crate::infrastructure::platform::authentication::session::FredSessionStore;
use crate::infrastructure::platform::authorization::principals::CacheBackedPrincipalLoader;
use crate::infrastructure::platform::identity::{
    PostgresPrincipalLoader, PostgresRefreshTokenStore, PostgresUserRepository,
};

pub type RegisterHandler =
    register_user::RegisterHandler<PostgresTransactionManager, PostgresUserRepository>;

pub type LoginHandler = login::LoginHandler<PostgresTransactionManager, PostgresUserRepository>;

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
    PostgresRefreshTokenStore,
>;

/// Concrete application handler type for refresh-token rotation.
pub type RotateRefreshTokenHandler = rotate_refresh_token::RefreshTokenHandler<
    PostgresTransactionManager,
    PostgresUserRepository,
    PostgresRefreshTokenStore,
>;

/// Concrete application handler type for refresh-token revocation.
pub type RevokeRefreshTokenHandler =
    revoke_refresh_token::RevokeTokenHandler<PostgresTransactionManager, PostgresRefreshTokenStore>;

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

/// Platform auth workflows exposed to the API layer.
#[derive(Clone)]
pub struct PlatformAuthHandlers {
    /// Registers a new user.
    pub register_user: Arc<RegisterHandler>,

    /// Authenticates a user using credentials.
    pub login: Arc<LoginHandler>,

    /// Logs out the current user.
    pub logout: Arc<LogoutHandler>,

    /// Issues an access token and refresh token using credentials.
    pub issue_access_token: Arc<IssueAccessTokenHandler>,

    /// Rotates a refresh token into a new token pair.
    pub rotate_refresh_token: Arc<RotateRefreshTokenHandler>,

    /// Revokes a refresh token.
    pub revoke_refresh_token: Arc<RevokeRefreshTokenHandler>,
}

/// Platform "me" workflows exposed to the API layer.
#[derive(Clone)]
pub struct PlatformMeHandlers {
    /// Changes the authenticated user's password.
    pub change_my_password: Arc<ChangeMyPasswordHandler>,
}

/// Platform workflows exposed to the API layer.
#[derive(Clone)]
pub struct PlatformHandlers {
    /// Authentication/token workflows.
    pub auth: PlatformAuthHandlers,

    /// Authenticated self-service account workflows.
    pub me: PlatformMeHandlers,
}

/// Platform-wide capabilities shared across application modules.
///
/// These are cross-cutting concerns that support the whole system.
#[derive(Clone)]
pub struct PlatformState {
    /// Request-boundary authentication components.
    pub authentication: Authentication,

    /// Session storage infrastructure.
    pub sessions: Sessions,

    /// Prewired platform workflows.
    pub handlers: PlatformHandlers,
}

/// Application-module state exposed to the API layer.
///
/// As the super-app grows, additional modules such as `kitchen` and `workout`
/// should be added here.
#[derive(Clone)]
pub struct AppsState {
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
    /// Cross-cutting platform capabilities and workflows.
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

impl FromRef<AppState> for PlatformHandlers {
    fn from_ref(state: &AppState) -> Self {
        state.platform.handlers.clone()
    }
}

impl FromRef<AppState> for PlatformAuthHandlers {
    fn from_ref(state: &AppState) -> Self {
        state.platform.handlers.auth.clone()
    }
}

impl FromRef<AppState> for PlatformMeHandlers {
    fn from_ref(state: &AppState) -> Self {
        state.platform.handlers.me.clone()
    }
}
