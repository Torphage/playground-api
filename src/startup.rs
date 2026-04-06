//! Application bootstrap and composition root.
//!
//! This module initializes infrastructure resources, assembles application
//! dependencies, and constructs the HTTP router.

use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use tokio::net::TcpListener;

use crate::api::router;
use crate::api::state::{
    AccountsAuthHandlers, AccountsHandlers, AccountsMeHandlers, AppState, AppsState,
    Authentication, ChangeMyPasswordHandler, IssueAccessTokenHandler, PlatformState,
    RevokeRefreshTokenHandler, RotateRefreshTokenHandler, Sessions, TokenIssuance,
};
use crate::application::authorization::Authorizer;
use crate::application::ports::{RefreshTokenService, TokenGenerator};
use crate::config::AppConfig;
use crate::domain::accounts::PasswordHasher;
use crate::infrastructure::authentication::refresh_token_service::DefaultRefreshTokenService;
use crate::infrastructure::authentication::session::{
    FredSessionStore, SessionRequestAuthenticator,
};
use crate::infrastructure::authentication::{
    CompositeRequestAuthenticator,
    jwt::{JwtProvider, JwtRequestAuthenticator, JwtVerifier},
};
use crate::infrastructure::authorization::permission_authorizer::PermissionAuthorizer;
use crate::infrastructure::crypto::Argon2Provider;
use crate::infrastructure::db::postgres::{
    PostgresTransactionManager, build_postgres_pool, run_postgres_migrations,
};
use crate::infrastructure::db::redis::{RedisClient, build_redis_client};
use crate::infrastructure::repositories::accounts::principals::{
    CacheBackedPrincipalLoader, RedisPrincipalCache,
};
use crate::infrastructure::repositories::accounts::{
    PostgresPrincipalLoader, PostgresRefreshTokenRepository, PostgresUserRepository,
};

type PrincipalLoader = CacheBackedPrincipalLoader<PostgresPrincipalLoader>;

/// Errors that can occur while bootstrapping the application.
#[derive(Debug)]
pub enum StartupError {
    PostgresPool(String),
    PostgresMigrations(String),
    RedisClient(String),
    TcpBind { address: String, message: String },
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PostgresPool(message) => {
                write!(f, "failed to initialize PostgreSQL pool: {message}")
            }
            Self::PostgresMigrations(message) => {
                write!(f, "failed to run PostgreSQL migrations: {message}")
            }
            Self::RedisClient(message) => {
                write!(f, "failed to initialize Redis client: {message}")
            }
            Self::TcpBind { address, message } => {
                write!(f, "failed to bind TCP listener on {address}: {message}")
            }
        }
    }
}

impl StdError for StartupError {}

/// Shared application components built once and reused across module builders.
///
/// This stays private to the composition root. It exists to prevent each module
/// (`accounts`, `kitchen`, `workout`, etc.) from rebuilding the same low-level
/// dependencies independently.
struct SharedComponents {
    tx_manager: PostgresTransactionManager,
    password_hasher: Arc<dyn PasswordHasher>,
    authorizer: Arc<dyn Authorizer>,
    principal_loader: Arc<PrincipalLoader>,
    user_repo: Arc<PostgresUserRepository>,
    refresh_token_repo: Arc<PostgresRefreshTokenRepository>,
    token_generator: Arc<dyn TokenGenerator>,
    refresh_token_service: Arc<dyn RefreshTokenService>,
}

/// Builds the infrastructure graph and returns the bound listener plus router.
///
/// Assumptions:
/// - `config.startup.run_migrations` exists and controls migration execution.
/// - `config.authentication.principal_cache_ttl_seconds` exists and controls
///   the principal cache TTL.
pub async fn build_application(config: AppConfig) -> Result<(TcpListener, Router), StartupError> {
    let config = Arc::new(config);

    let pool = build_postgres_pool(&config.database.url)
        .await
        .map_err(|e| StartupError::PostgresPool(e.to_string()))?;

    run_migrations_if_enabled(&config, &pool).await?;

    let redis_client = build_redis_client(&config.redis)
        .await
        .map_err(|e| StartupError::RedisClient(e.to_string()))?;

    let state = build_state(pool, redis_client, Arc::clone(&config));
    let router = router::create_router(state, config.cors.clone());
    let listener = build_listener(&config).await?;

    Ok((listener, router))
}

/// Runs PostgreSQL migrations when enabled in configuration.
async fn run_migrations_if_enabled(
    config: &Arc<AppConfig>,
    pool: &sqlx::PgPool,
) -> Result<(), StartupError> {
    if !config.startup.run_migrations {
        return Ok(());
    }

    tracing::info!("Running database migrations...");

    run_postgres_migrations(pool)
        .await
        .map_err(|e| StartupError::PostgresMigrations(e.to_string()))
}

/// Binds the TCP listener.
async fn build_listener(config: &AppConfig) -> Result<TcpListener, StartupError> {
    let address = config.server.bind_address();

    TcpListener::bind(&address)
        .await
        .map_err(|e| StartupError::TcpBind {
            address,
            message: e.to_string(),
        })
}

/// Constructs the dependency container used by request handlers.
fn build_state(pool: sqlx::PgPool, redis_client: RedisClient, config: Arc<AppConfig>) -> AppState {
    let sessions = build_sessions(redis_client.clone(), &config);
    let authentication = build_authentication(&config, Arc::clone(&sessions.store));
    let shared = build_shared_components(pool, redis_client, &config);

    let platform = build_platform(
        authentication,
        sessions,
        shared.token_generator.clone(),
        shared.refresh_token_service.clone(),
    );

    let accounts = build_accounts(&shared, &config);

    AppState {
        platform,
        apps: AppsState { accounts },
        config,
    }
}

/// Builds session-related dependencies.
fn build_sessions(redis_client: RedisClient, config: &AppConfig) -> Sessions {
    Sessions {
        store: Arc::new(FredSessionStore::new(
            redis_client,
            Duration::from_secs(config.authentication.session.ttl_seconds),
        )),
    }
}

/// Builds request-boundary authentication dependencies.
///
/// Authentication order is intentional:
/// 1. JWT bearer authentication
/// 2. Session-cookie authentication
fn build_authentication(
    config: &AppConfig,
    session_store: Arc<FredSessionStore>,
) -> Authentication {
    let jwt_verifier = JwtVerifier::new(&config.authentication.jwt);
    let jwt_request_authenticator = Arc::new(JwtRequestAuthenticator::new(jwt_verifier));

    let session_request_authenticator = Arc::new(SessionRequestAuthenticator::new(
        session_store,
        config.authentication.session.cookie_name.clone(),
    ));

    let request_authenticator = CompositeRequestAuthenticator::new(vec![
        jwt_request_authenticator,
        session_request_authenticator,
    ]);

    Authentication {
        request_authenticator: Arc::new(request_authenticator),
    }
}

fn build_platform(
    authentication: Authentication,
    sessions: Sessions,
    token_generator: Arc<dyn TokenGenerator>,
    refresh_token_service: Arc<dyn RefreshTokenService>,
) -> PlatformState {
    PlatformState {
        authentication,
        sessions,
        token_issuance: TokenIssuance {
            token_generator,
            refresh_token_service,
        },
    }
}

/// Builds shared low-level application dependencies once.
///
/// These are reused by module-specific builders so that each module can
/// prewire its own workflows without duplicating infrastructure construction.
fn build_shared_components(
    pool: sqlx::PgPool,
    redis_client: RedisClient,
    config: &AppConfig,
) -> SharedComponents {
    let tx_manager = PostgresTransactionManager::new(pool);

    let postgres_principal_loader = Arc::new(PostgresPrincipalLoader::new());
    let redis_principal_cache = Arc::new(RedisPrincipalCache::new(
        redis_client,
        config.authentication.principal_cache_ttl_seconds,
    ));
    let principal_loader = Arc::new(CacheBackedPrincipalLoader::new(
        postgres_principal_loader,
        redis_principal_cache,
    ));

    let user_repo = Arc::new(PostgresUserRepository::new());
    let refresh_token_repo = Arc::new(PostgresRefreshTokenRepository::new());

    let password_hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2Provider::new());
    let authorizer: Arc<dyn Authorizer> = Arc::new(PermissionAuthorizer::new());
    let token_generator: Arc<dyn TokenGenerator> =
        Arc::new(JwtProvider::new(&config.authentication.jwt));
    let refresh_token_service: Arc<dyn RefreshTokenService> =
        Arc::new(DefaultRefreshTokenService::new());

    SharedComponents {
        tx_manager,
        password_hasher,
        authorizer,
        principal_loader,
        user_repo,
        refresh_token_repo,
        token_generator,
        refresh_token_service,
    }
}

fn build_accounts(shared: &SharedComponents, config: &AppConfig) -> AccountsHandlers {
    AccountsHandlers {
        auth: build_accounts_auth_handlers(shared, config),
        me: build_accounts_me_handlers(shared),
    }
}

fn build_accounts_auth_handlers(
    shared: &SharedComponents,
    config: &AppConfig,
) -> AccountsAuthHandlers {
    let issue_token = Arc::new(IssueAccessTokenHandler::new(
        shared.tx_manager.clone(),
        shared.user_repo.clone(),
        shared.refresh_token_repo.clone(),
        shared.password_hasher.clone(),
        shared.token_generator.clone(),
        shared.refresh_token_service.clone(),
        config.authentication.jwt.refresh_ttl_seconds,
    ));

    let refresh_token = Arc::new(RotateRefreshTokenHandler::new(
        shared.tx_manager.clone(),
        shared.user_repo.clone(),
        shared.refresh_token_repo.clone(),
        shared.token_generator.clone(),
        shared.refresh_token_service.clone(),
        config.authentication.jwt.refresh_ttl_seconds,
    ));

    let revoke_token = Arc::new(RevokeRefreshTokenHandler::new(
        shared.tx_manager.clone(),
        shared.refresh_token_repo.clone(),
        shared.refresh_token_service.clone(),
    ));

    AccountsAuthHandlers {
        issue_token,
        refresh_token,
        revoke_token,
    }
}

fn build_accounts_me_handlers(shared: &SharedComponents) -> AccountsMeHandlers {
    let change_my_password = Arc::new(ChangeMyPasswordHandler::new(
        shared.tx_manager.clone(),
        shared.user_repo.clone(),
        shared.principal_loader.clone(),
        shared.password_hasher.clone(),
        shared.authorizer.clone(),
    ));

    AccountsMeHandlers { change_my_password }
}
