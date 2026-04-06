//! Application-level transaction contracts.
//!
//! This module defines the minimal abstractions required by the application
//! layer to execute use cases inside a transactional boundary without leaking
//! a concrete database driver such as `sqlx` outside infrastructure.
//!
//! The transaction type is generic over a lifetime so that infrastructure can
//! wrap standard SQLx transactions safely and idiomatically.

use async_trait::async_trait;

use crate::application::error::AppError;

/// Represents an in-flight transaction controlled by the application layer.
///
/// Concrete implementations may wrap database-specific transaction types such as
/// `sqlx::Transaction<'a, Postgres>`.
#[async_trait]
pub trait Transaction: Send {
    /// Commits the transaction and makes all prior operations durable.
    async fn commit(self) -> Result<(), AppError>;

    /// Rolls the transaction back, discarding all changes made within it.
    async fn rollback(self) -> Result<(), AppError>;
}

/// Starts new transactions for application workflows.
///
/// The associated transaction type is lifetime-aware so that infrastructure can
/// expose standard SQLx transaction wrappers without unsafe workarounds.
#[async_trait]
pub trait TransactionManager: Send + Sync {
    /// The concrete transaction type returned by this manager for a given lifetime.
    type Tx<'a>: Transaction
    where
        Self: 'a;

    /// Begins a new transaction.
    async fn begin<'a>(&'a self) -> Result<Self::Tx<'a>, AppError>;
}
