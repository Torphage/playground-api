//! PostgreSQL pool bootstrap helpers.
//!
//! This module owns shared SQLx pool creation and migration execution.

use sqlx::PgPool;

use crate::application::error::AppError;

/// Builds the shared PostgreSQL connection pool.
pub async fn build_postgres_pool(database_url: &str) -> Result<PgPool, AppError> {
    PgPool::connect(database_url)
        .await
        .map_err(|e| AppError::Infrastructure(format!("Failed to connect to Postgres: {e}")))
}

/// Runs SQLx migrations using the provided PostgreSQL pool.
pub async fn run_postgres_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::Infrastructure(format!("Failed to migrate DB: {e}")))
}
