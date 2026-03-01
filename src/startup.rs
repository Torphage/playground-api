// src/startup.rs
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::api;
use crate::api::state::{AppState, Repositories};
use crate::config::Config;
use crate::infrastructure::repositories::identity::users::postgres::PostgresUserRepository;

// The monolithic setup function
pub async fn build_application(config: Config) -> Result<(TcpListener, Router), String> {
    // Setup Database Connection
    let pool = sqlx::PgPool::connect(&config.database.url)
        .await
        .map_err(|e| format!("Failed to connect to Postgres: {}", e))?;

    // Run Migrations (Optional but recommended)
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Failed to migrate DB: {}", e))?;

    let state = build_state(pool, config.clone()).await;

    // Build the Router
    let router = api::router::create_router(state, config.cors);

    // Bind the Listener
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&address)
        .await
        .map_err(|e| format!("Failed to bind to {}: {}", address, e))?;

    Ok((listener, router))
}

/// Initialize the application state
async fn build_state(pool: sqlx::PgPool, config: Config) -> AppState {
    // Instantiate Repositories
    let repos = Repositories {
        user: Arc::new(PostgresUserRepository),
    };

    // Build State
    AppState {
        repos,
        pool,
        config: Arc::new(config),
    }
}
