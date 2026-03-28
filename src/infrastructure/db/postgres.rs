//! PostgreSQL transaction infrastructure.
//!
//! This module adapts PostgreSQL connection and transaction handling to the
//! application's generic transaction contracts.
//!
//! # Design
//!
//! We intentionally avoid storing `sqlx::Transaction<'_ , Postgres>` here.
//! While SQLx transactions are ergonomic inside infrastructure code, wrapping
//! them into an application-facing owned type introduces lifetime complexity.
//!
//! Instead, this adapter:
//! - acquires an owned pooled PostgreSQL connection
//! - starts a transaction explicitly using `BEGIN`
//! - commits using `COMMIT`
//! - rolls back using `ROLLBACK`
//!
//! This keeps the implementation fully safe, explicit, and free of `unsafe`.

use async_trait::async_trait;
use sqlx::{pool::PoolConnection, Connection, Executor, PgPool, Postgres};

use crate::application::AppError;
use crate::application::ports::{Transaction, TransactionManager};

/// Starts PostgreSQL transactions backed by a shared SQLx pool.
///
/// Each call to [`begin`](TransactionManager::begin) acquires a dedicated
/// connection from the pool and places that connection into an explicit
/// transaction scope.
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

/// A PostgreSQL transaction backed by a dedicated pooled connection.
///
/// This type represents a single in-flight database transaction. It exposes the
/// underlying connection to infrastructure repositories so that all queries in
/// a use case participate in the same transaction.
///
/// Transaction completion is explicit:
/// - [`commit`](Transaction::commit) persists the changes
/// - [`rollback`](Transaction::rollback) discards them
///
/// No implicit rollback is performed on drop in this implementation.
pub struct PostgresTransaction {
    connection: PoolConnection<Postgres>,
}

impl PostgresTransaction {
    /// Creates a new transaction wrapper from a pooled PostgreSQL connection.
    fn new(connection: PoolConnection<Postgres>) -> Self {
        Self { connection }
    }

    /// Returns a mutable reference to the underlying PostgreSQL connection.
    ///
    /// This is intended for infrastructure repository implementations only.
    /// All SQL executed through this connection participates in the active
    /// transaction started by the transaction manager.
    pub fn connection_mut(&mut self) -> &mut PoolConnection<Postgres> {
        &mut self.connection
    }
}

#[async_trait]
impl Transaction for PostgresTransaction {
    /// Commits the active transaction.
    async fn commit(mut self) -> Result<(), AppError> {
        sqlx::query("COMMIT")
            .execute(self.connection_mut())
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to commit transaction: {e}")))?;

        Ok(())
    }

    /// Rolls back the active transaction.
    async fn rollback(mut self) -> Result<(), AppError> {
        sqlx::query("ROLLBACK")
            .execute(self.connection_mut())
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to roll back transaction: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl TransactionManager for PostgresTransactionManager {
    type Tx = PostgresTransaction;

    /// Acquires a pooled PostgreSQL connection and starts a new transaction.
    async fn begin(&self) -> Result<Self::Tx, AppError> {
        let mut connection = self
            .pool
            .acquire()
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to acquire database connection: {e}")))?;

        connection
            .execute("BEGIN")
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to begin transaction: {e}")))?;

        Ok(PostgresTransaction::new(connection))
    }
}