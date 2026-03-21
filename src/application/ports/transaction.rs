//! Application-level transaction contracts.
//!
//! This module defines the minimal abstractions required by the application
//! layer to execute use cases inside a transactional boundary without leaking
//! a concrete database driver such as `sqlx` outside infrastructure.
//!
//! The design is intentionally small:
//! - `Transaction` models an in-flight transaction
//! - `TransactionManager` starts new transactions
//!
//! Concrete implementations belong to the infrastructure layer.

use async_trait::async_trait;

use crate::application::error::AppError;

/// Represents an in-flight transaction controlled by the application layer.
///
/// Implementations are expected to wrap concrete backend transaction/session
/// primitives such as a PostgreSQL transaction or a MongoDB session.
///
/// # Rollback semantics
/// Implementations should prefer fail-safe behavior:
/// - `commit()` finalizes the transaction
/// - `rollback()` aborts it explicitly
/// - dropping an uncommitted transaction should roll it back if supported by
///   the underlying driver
#[async_trait]
pub trait Transaction: Send {
    /// Commits the transaction and makes all prior operations durable.
    async fn commit(self) -> Result<(), AppError>;

    /// Rolls the transaction back, discarding all changes made within it.
    async fn rollback(self) -> Result<(), AppError>;
}

/// Starts new transactions for application workflows.
///
/// This trait is owned by the application boundary and implemented by
/// infrastructure-specific adapters.
#[async_trait]
pub trait TransactionManager: Send + Sync {
    /// The concrete transaction type returned by this manager.
    type Tx: Transaction;

    /// Begins a new transaction.
    async fn begin(&self) -> Result<Self::Tx, AppError>;
}