// src/startup.rs
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::api;
use crate::api::{AppState, Crypto, Repositories};
use crate::config::AppConfig;
use crate::infrastructure::crypto::Argon2Provider;
use crate::infrastructure::db::PostgresTransactionManager;
use crate::infrastructure::repositories::identity::PostgresUserRepository;

/// Builds the application listener and router.
///
/// This function is the composition root of the service. It wires concrete
/// infrastructure implementations into the HTTP layer.
pub async fn build_application(config: Config) -> Result<(TcpListener, Router), String> {
    let pool = sqlx::PgPool::connect(&config.database.url)
        .await
        .map_err(|e| format!("Failed to connect to Postgres: {e}"))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to migrate DB: {e}"))?;

    let state = build_state(pool, config.clone()).await;

    let router = api::router::create_router(state, config.cors);

    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&address)
        .await
        .map_err(|e| format!("Failed to bind to {address}: {e}"))?;

    Ok((listener, router))
}

/// Initializes the shared application state.
async fn build_state(pool: sqlx::PgPool, config: Config) -> AppState {
    let repos = Repositories {
        user: Arc::new(PostgresUserRepository::new()),
    };

    let crypto = Crypto {
        password_hasher: Arc::new(Argon2Provider::new()),
    };

    let tx_manager = PostgresTransactionManager::new(pool);

    AppState {
        repos,
        tx_manager,
        crypto,
        config: Arc::new(config),
    }
}