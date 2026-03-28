//! Database infrastructure adapters.
//!
//! This module contains concrete database integration code such as connection
//! pools, transaction managers, and backend-specific transaction wrappers.

mod postgres;

pub use postgres::{PostgresTransaction, PostgresTransactionManager};
