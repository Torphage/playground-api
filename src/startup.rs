//! Application bootstrap and composition root.
//!
//! This module handles the initialization of infrastructure resources (database
//! pools, external clients) and wires them into the application state prior
//! to binding the HTTP listener.

use axum::Router;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::api;
use crate::api::{AppState, Crypto, Repositories};
use crate::config::AppConfig;
use crate::infrastructure::crypto::Argon2Provider;
use crate::infrastructure::db::PostgresTransactionManager;
use crate::infrastructure::repositories::identity::PostgresUserRepository;

/// Assembles infrastructure dependencies and constructs the routing tree.
pub async fn build_application(config: AppConfig) -> Result<(TcpListener, Router), String> {
    let pool = sqlx::PgPool::connect(&config.database.url)
        .await
        .map_err(|e| format!("Failed to connect to Postgres: {e}"))?;

    // Database migrations are gated to prevent concurrent execution conflicts
    // when scaling multiple API instances in a production cluster.
    if env::var("RUN_MIGRATIONS").unwrap_or_default() == "true" {
        tracing::info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| format!("Failed to migrate DB: {e}"))?;
    }

    let state = build_state(pool, config.clone()).await;

    let router = api::router::create_router(state, config.cors);

    let address = config.server.bind_address();
    let listener = TcpListener::bind(&address)
        .await
        .map_err(|e| format!("Failed to bind to {address}: {e}"))?;

    Ok((listener, router))
}

/// Constructs the dependency injection container for HTTP handlers.
async fn build_state(pool: sqlx::PgPool, config: AppConfig) -> AppState {
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
