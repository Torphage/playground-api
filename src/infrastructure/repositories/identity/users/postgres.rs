//! PostgreSQL implementation of the UserRepository with RBAC support.
//!
//! This module fulfills the storage contracts required by the authentication
//! domain. It interacts with the `auth.users`, `auth.roles`, `auth.user_roles`,
//! and `auth.role_permissions` tables, guaranteeing that the rich `User`
//! aggregate root is always persisted and reconstructed in a fully consistent state.

use async_trait::async_trait;
use crate::application::error::AppError;
use crate::domain::identity::entities::user::User;
use crate::domain::identity::ports::user_repository::UserRepository;
use crate::domain::identity::values::email::Email;
use crate::domain::identity::values::user_id::UserId;

use super::rows::{RoleRow, UserPermissionRow, UserRow};
use super::mapper::assemble_user;

/// A PostgreSQL-backed repository for the `User` aggregate.
pub struct PostgresUserRepository;

impl PostgresUserRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    /// Persists a user and their assigned roles to the database.
    ///
    /// Because `User` is an aggregate root, saving it means synchronizing its
    /// entire state. This method upserts the core user data and entirely
    /// replaces their assigned roles in the junction table to match the
    /// aggregate's current state in memory.
    async fn save(&self, conn: &mut sqlx::PgConnection, user: &User) -> Result<(), AppError> {
        let user_uuid = user.id.as_uuid();

        // 1. Upsert the core user record
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
            .execute(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to save user core: {}", e)))?;

        // 2. Synchronize Roles (Clear and Re-insert)
        // This guarantees the database exactly matches the aggregate root.
        sqlx::query!(
            r#"DELETE FROM identity.user_roles WHERE user_id = $1"#,
            user_uuid
        )
            .execute(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to clear user roles: {}", e)))?;

        for role in &user.roles {
            sqlx::query!(
                r#"
                INSERT INTO identity.user_roles (user_id, role_id)
                VALUES ($1, $2)
                "#,
                user_uuid,
                role.id
            )
                .execute(conn)
                .await
                .map_err(|e| AppError::Infrastructure(format!("Failed to insert user role: {}", e)))?;
        }

        Ok(())
    }

    /// Retrieves a user by their unique identifier, fully loaded with RBAC data.
    async fn find_by_id(&self, conn: &mut sqlx::PgConnection, id: &UserId) -> Result<Option<User>, AppError> {
        let user_uuid = id.as_uuid();

        // 1. Fetch the core user
        let user_row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, email, password_hash, created_at, updated_at FROM identity.users WHERE id = $1"#,
            user_uuid
        )
            .fetch_optional(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Query failed: {}", e)))?;

        let user_row = match user_row {
            Some(row) => row,
            None => return Ok(None),
        };

        // 2. Fetch assigned roles
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
            .fetch_all(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch roles: {}", e)))?;

        // 3. Fetch flattened permissions via JOIN
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
            .fetch_all(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch permissions: {}", e)))?;

        // 4. Hand off the raw rows to the assembler
        let user = assemble_user(user_row, role_rows, permission_rows)?;
        Ok(Some(user))
    }

    /// Retrieves a user by their email address, fully loaded with RBAC data.
    async fn find_by_email(&self, conn: &mut sqlx::PgConnection, email: &Email) -> Result<Option<User>, AppError> {
        // 1. Fetch the core user to get their UUID
        let user_row = sqlx::query_as!(
            UserRow,
            r#"SELECT id, email, password_hash, created_at, updated_at FROM identity.users WHERE email = $1"#,
            email.as_str()
        )
            .fetch_optional(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Query failed: {}", e)))?;

        let user_row = match user_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let user_uuid = user_row.id;

        // 2. Fetch assigned roles
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
            .fetch_all(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch roles: {}", e)))?;

        // 3. Fetch flattened permissions
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
            .fetch_all(conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to fetch permissions: {}", e)))?;

        // 4. Assemble and return
        let user = assemble_user(user_row, role_rows, permission_rows)?;
        Ok(Some(user))
    }
}
