//! Application bootstrap and composition root.
//!
//! This module handles the initialization of infrastructure resources (database
//! pools, external clients) and wires them into the application state prior
//! to binding the HTTP listener.

use axum::Router;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

use crate::api::{
    AppState, Authentication, Authorization, Crypto, Repositories, Sessions, TokenIssuance, router,
};
use crate::config::AppConfig;
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
use crate::infrastructure::db::redis::build_redis_client;
use crate::infrastructure::repositories::accounts::principals::{
    CacheBackedPrincipalLoader, RedisPrincipalCache,
};
use crate::infrastructure::repositories::accounts::{
    PostgresPrincipalLoader, PostgresUserRepository,
};

/// Assembles infrastructure dependencies and constructs the routing tree.
pub async fn build_application(config: AppConfig) -> Result<(TcpListener, Router), String> {
    let config = Arc::new(config);

    let pool = build_postgres_pool(&config.database.url)
        .await
        .map_err(|e| e.to_string())?;

    if env::var("RUN_MIGRATIONS").unwrap_or_default() == "true" {
        tracing::info!("Running database migrations...");
        run_postgres_migrations(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    let redis_client = build_redis_client(&config.redis)
        .await
        .map_err(|e| e.to_string())?;

    let state = build_state(pool, redis_client, config.clone());

    let router = router::create_router(state, config.cors.clone());

    let address = config.server.bind_address();
    let listener = TcpListener::bind(&address)
        .await
        .map_err(|e| format!("Failed to bind to {address}: {e}"))?;

    Ok((listener, router))
}

/// Constructs the dependency injection container for HTTP handlers.
fn build_state(
    pool: sqlx::PgPool,
    redis_client: crate::infrastructure::db::redis::RedisClient,
    config: Arc<AppConfig>,
) -> AppState {
    let postgres_principal_loader = Arc::new(PostgresPrincipalLoader::new());
    let redis_principal_cache = Arc::new(RedisPrincipalCache::new(redis_client.clone(), 300));
    let principal_loader = Arc::new(CacheBackedPrincipalLoader::new(
        postgres_principal_loader,
        redis_principal_cache,
    ));

    let repos = Repositories {
        principal: principal_loader,
        user: Arc::new(PostgresUserRepository::new()),
    };

    let crypto = Crypto {
        password_hasher: Arc::new(Argon2Provider::new()),
    };

    let session_store = Arc::new(FredSessionStore::new(
        redis_client,
        Duration::from_secs(config.authentication.session.ttl_seconds),
    ));

    let jwt_verifier = JwtVerifier::new(&config.authentication.jwt);
    let jwt_request_authenticator = Arc::new(JwtRequestAuthenticator::new(jwt_verifier));

    let session_request_authenticator = Arc::new(SessionRequestAuthenticator::new(
        session_store.clone(),
        config.authentication.session.cookie_name.clone(),
    ));

    let request_authenticator = CompositeRequestAuthenticator::new()
        .push(jwt_request_authenticator)
        .push(session_request_authenticator);

    let authentication = Authentication {
        request_authenticator: Arc::new(request_authenticator),
    };

    let sessions = Sessions {
        store: session_store,
    };

    let token_issuance = TokenIssuance {
        token_generator: Arc::new(JwtProvider::new(&config.authentication.jwt)),
    };

    let authorization = Authorization {
        authorizer: Arc::new(PermissionAuthorizer::new()),
    };

    let tx_manager = PostgresTransactionManager::new(pool);

    AppState {
        repos,
        tx_manager,
        crypto,
        authentication,
        sessions,
        token_issuance,
        authorization,
        config,
    }
}
