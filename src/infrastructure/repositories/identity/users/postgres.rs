//! PostgreSQL implementation of the `UserRepository` port.
//!
//! This module fulfills the storage contract for the `User` aggregate using
//! PostgreSQL and SQLx. It is responsible for translating repository operations
//! into concrete SQL queries while keeping those details isolated from the
//! application and domain layers.

use async_trait::async_trait;

use crate::application::AppError;
use crate::domain::identity::{
    entities::User,
    ports::UserRepository,
    values::{Email, UserId},
};
use crate::infrastructure::db::PostgresTransaction;

use super::mapper::assemble_user;
use super::rows::{RoleRow, UserPermissionRow, UserRow};

/// A PostgreSQL-backed repository for the `User` aggregate.
#[derive(Default)]
pub struct PostgresUserRepository;

impl PostgresUserRepository {
    /// Creates a new repository instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UserRepository<PostgresTransaction> for PostgresUserRepository {
    /// Persists a user aggregate and synchronizes its assigned roles.
    ///
    /// Because `User` is treated as an aggregate root, saving it means
    /// synchronizing the full persistence representation required by the
    /// aggregate, including the user-role relationship table.
    async fn save(&self, tx: &mut PostgresTransaction, user: &User) -> Result<(), AppError> {
        let user_uuid = user.id.as_uuid();
        let conn = tx.connection_mut();

        // 1. Upsert the core user record.
        sqlx::query!(
            r#"
            INSERT INTO identity.users (id, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (id) DO UPDATE
            SET email = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                updated_at = EXCLUDED.updated_at
            "#,
            user_uuid,
            user.email.as_str(),
            user.password_hash.as_str(),
            user.created_at,
            user.updated_at
        )
            .execute(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to save user core: {e}")))?;

        // 2. Replace role assignments to match the aggregate's current state.
        sqlx::query!(
            r#"DELETE FROM identity.user_roles WHERE user_id = $1"#,
            user_uuid
        )
            .execute(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to clear user roles: {e}")))?;

        for role in &user.roles {
            sqlx::query!(
                r#"
                INSERT INTO identity.user_roles (user_id, role_id)
                VALUES ($1, $2)
                "#,
                user_uuid,
                role.id
            )
                .execute(&mut **conn)
                .await
                .map_err(|e| AppError::Infrastructure(format!("Failed to insert user role: {e}")))?;
        }

        Ok(())
    }

    /// Retrieves a fully assembled user by identifier.
    async fn find_by_id(
        &self,
        tx: &mut PostgresTransaction,
        id: &UserId,
    ) -> Result<Option<User>, AppError> {
        let user_uuid = id.as_uuid();
        let conn = tx.connection_mut();

        // 1. Fetch the core user row.
        let user_row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, email, password_hash, created_at, updated_at FROM identity.users WHERE id = $1"#,
            user_uuid
        )
            .fetch_optional(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Query failed: {e}")))?;

        let user_row = match user_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // 2. Fetch assigned roles.
        let role_rows = sqlx::query_as!(
            RoleRow,
            r#"
            SELECT r.id, r.name
            FROM identity.roles r
            JOIN identity.user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_uuid
        )
            .fetch_all(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch roles: {e}")))?;

        // 3. Fetch flattened permissions granted through roles.
        let permission_rows = sqlx::query_as!(
            UserPermissionRow,
            r#"
            SELECT rp.permission_slug
            FROM identity.role_permissions rp
            JOIN identity.user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_uuid
        )
            .fetch_all(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch permissions: {e}")))?;

        // 4. Reconstruct the aggregate.
        let user = assemble_user(user_row, role_rows, permission_rows)?;
        Ok(Some(user))
    }

    /// Retrieves a fully assembled user by email address.
    async fn find_by_email(
        &self,
        tx: &mut PostgresTransaction,
        email: &Email,
    ) -> Result<Option<User>, AppError> {
        let conn = tx.connection_mut();

        // Fetch the core user row.
        let user_row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, email, password_hash, created_at, updated_at FROM identity.users WHERE email = $1"#,
            email.as_str()
        )
            .fetch_optional(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Query failed: {e}")))?;

        let user_row = match user_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let user_uuid = user_row.id;

        // Fetch assigned roles.
        let role_rows = sqlx::query_as!(
            RoleRow,
            r#"
            SELECT r.id, r.name
            FROM identity.roles r
            JOIN identity.user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_uuid
        )
            .fetch_all(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch roles: {e}")))?;

        // Fetch flattened permissions.
        let permission_rows = sqlx::query_as!(
            UserPermissionRow,
            r#"
            SELECT rp.permission_slug
            FROM identity.role_permissions rp
            JOIN identity.user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_uuid
        )
            .fetch_all(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch permissions: {e}")))?;

        // Reconstruct and return the aggregate.
        let user = assemble_user(user_row, role_rows, permission_rows)?;
        Ok(Some(user))
    }
}
