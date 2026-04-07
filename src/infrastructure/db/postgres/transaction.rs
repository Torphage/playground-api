//! PostgreSQL transaction infrastructure.
//!
//! This module adapts standard SQLx PostgreSQL transactions to the application's
//! generic transaction contracts.
//!
//! The transaction wrapper is lifetime-aware, which is the normal and safe SQLx
//! model. The lifetime stays hidden behind infrastructure as much as possible.

use async_trait::async_trait;
use sqlx::{PgPool, Postgres};

use crate::application::error::AppError;
use crate::application::shared::{Transaction, TransactionManager};

/// Starts PostgreSQL transactions backed by a shared SQLx pool.
#[derive(Clone)]
pub struct PostgresTransactionManager {
    pool: PgPool,
}

impl PostgresTransactionManager {
    /// Creates a new PostgreSQL transaction manager from the provided pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// A PostgreSQL transaction backed by SQLx.
///
/// The lifetime ties the transaction to the owning connection/pool borrow in a
/// way that SQLx understands and enforces safely.
pub struct PostgresTransaction<'a> {
    inner: sqlx::Transaction<'a, Postgres>,
}

impl<'a> PostgresTransaction<'a> {
    /// Wraps a standard SQLx PostgreSQL transaction.
    fn new(inner: sqlx::Transaction<'a, Postgres>) -> Self {
        Self { inner }
    }

    /// Returns the underlying SQLx transaction mutably.
    pub fn as_mut(&mut self) -> &mut sqlx::Transaction<'a, Postgres> {
        &mut self.inner
    }
}

#[async_trait]
impl<'a> Transaction for PostgresTransaction<'a> {
    /// Commits the active transaction.
    async fn commit(self) -> Result<(), AppError> {
        self.inner
            .commit()
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to commit transaction: {e}")))
    }

    /// Rolls back the active transaction.
    async fn rollback(self) -> Result<(), AppError> {
        self.inner
            .rollback()
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to roll back transaction: {e}")))
    }
}

#[async_trait]
impl TransactionManager for PostgresTransactionManager {
    type Tx<'a>
        = PostgresTransaction<'a>
    where
        Self: 'a;

    /// Acquires a pooled connection and starts a new SQLx transaction.
    async fn begin<'a>(&'a self) -> Result<Self::Tx<'a>, AppError> {
        let tx =
            self.pool.begin().await.map_err(|e| {
                AppError::Infrastructure(format!("Failed to begin transaction: {e}"))
            })?;

        Ok(PostgresTransaction::new(tx))
    }
}
