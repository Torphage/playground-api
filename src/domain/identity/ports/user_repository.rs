//! User Repository Port.
//!
//! Defines the storage contracts for the User aggregate root. Implementations
//! of this trait must guarantee that the aggregate is always saved and
//! reconstructed in a consistent state.

use async_trait::async_trait;
use sqlx;
use crate::application::error::AppError;
use crate::domain::identity::entities::user::User;
use crate::domain::identity::values::email::Email;
use crate::domain::identity::values::user_id::UserId;

// =========================================================================
// REPOSITORY PORT
// =========================================================================

/// The outbound port for User persistence.
///
/// Notice that every method takes a `&mut UnitOfWork`. This enforces the
/// architectural rule that the repository does not manage its own database
/// connections; it borrows them from the active transaction context.
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Persists a user (and their associated RBAC roles) to the data store.
    ///
    /// Returns an `AuthError` if a business constraint is violated at the
    /// storage level, such as an email uniqueness conflict.
    async fn save(&self, &mut sqlx::PgConnection, user: &User) -> Result<(), AppError>;

    /// Retrieves a user by their unique Domain ID.
    ///
    /// Returns `Ok(None)` if no user matches the provided ID.
    async fn find_by_id(&self, conn: &mut sqlx::PgConnection, id: &UserId) -> Result<Option<User>, AppError>;

    /// Retrieves a user by their strongly-typed Domain Email.
    ///
    /// This is primarily utilized during the authentication/login process.
    async fn find_by_email(&self, conn: &mut sqlx::PgConnection, email: &Email) -> Result<Option<User>, AppError>;
}
