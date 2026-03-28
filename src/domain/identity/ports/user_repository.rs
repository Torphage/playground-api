//! User repository port.
//!
//! Defines the persistence contract for the `User` aggregate root.
//! Implementations are responsible for storing and reconstructing a user in a
//! consistent state, including any related authorization data required by the
//! aggregate.
//!
//! The repository is generic over a transaction type so that the application
//! layer can control transaction boundaries without leaking infrastructure
//! concerns such as `sqlx` into this trait.

use async_trait::async_trait;

use crate::application::AppError;
use crate::domain::identity::{
    entities::User,
    values::{Email, UserId},
};

/// The outbound persistence port for the `User` aggregate.
///
/// The transaction is supplied by the application layer so that all repository
/// calls participating in a single use case can share the same transactional
/// boundary.
#[async_trait]
pub trait UserRepository<Tx>: Send + Sync {
    /// Persists a user aggregate and its related authorization state.
    async fn save(&self, tx: &mut Tx, user: &User) -> Result<(), AppError>;

    /// Retrieves a user by domain identifier.
    async fn find_by_id(&self, tx: &mut Tx, id: &UserId) -> Result<Option<User>, AppError>;

    /// Retrieves a user by email address.
    async fn find_by_email(&self, tx: &mut Tx, email: &Email) -> Result<Option<User>, AppError>;
}
