//! PostgreSQL implementation of the `UserRepository` port.

use async_trait::async_trait;

use crate::application::error::AppError;
use crate::domain::identity::{
    entities::User,
    ports::UserRepository,
    values::{Email, UserId},
};
use crate::infrastructure::db::postgres::PostgresTransaction;
use crate::infrastructure::repositories::identity::users::assembly::assemble_user;

use super::mapper::map_user_rows;
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
impl<'a> UserRepository<PostgresTransaction<'a>> for PostgresUserRepository {
    /// Persists a user aggregate and synchronizes its assigned roles.
    async fn save(&self, tx: &mut PostgresTransaction<'a>, user: &User) -> Result<(), AppError> {
        let user_uuid = user.id.as_uuid();
        let conn = tx.as_mut();

        sqlx::query!(
            r#"
            INSERT INTO identity.users (id, username, email, password_hash, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET email = EXCLUDED.email,
                password_hash = EXCLUDED.password_hash,
                updated_at = EXCLUDED.updated_at
            "#,
            user_uuid,
            user.username.as_str(),
            user.email.as_str(),
            user.password_hash.as_str(),
            user.created_at,
            user.updated_at
        )
            .execute(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to save user core: {e}")))?;

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
        tx: &mut PostgresTransaction<'a>,
        id: &UserId,
    ) -> Result<Option<User>, AppError> {
        let user_uuid = id.as_uuid();
        let conn = tx.as_mut();

        let user_row: Option<UserRow> = sqlx::query_as!(
            UserRow,
            r#"SELECT id, username, email, password_hash, created_at, updated_at
               FROM identity.users WHERE id = $1"#,
            user_uuid
        )
            .fetch_optional(&mut **conn)
            .await
            .map_err(|e| AppError::Infrastructure(format!("Query failed: {e}")))?;

        let user_row = match user_row {
            Some(row) => row,
            None => return Ok(None),
        };

        let role_rows: Vec<RoleRow> = sqlx::query_as!(
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

        let permission_rows: Vec<UserPermissionRow> = sqlx::query_as!(
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

        let input = map_user_rows(user_row, role_rows, permission_rows);
        let user = assemble_user(input)?;
        Ok(Some(user))
    }

    /// Retrieves a fully assembled user by email address.
    async fn find_by_email(
        &self,
        tx: &mut PostgresTransaction<'a>,
        email: &Email,
    ) -> Result<Option<User>, AppError> {
        let conn = tx.as_mut();

        let user_row: Option<UserRow> = sqlx::query_as!(
            UserRow,
            r#"SELECT id, username, email, password_hash, created_at, updated_at
               FROM identity.users WHERE email = $1"#,
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

        let role_rows: Vec<RoleRow> = sqlx::query_as!(
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

        let permission_rows: Vec<UserPermissionRow> = sqlx::query_as!(
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

        let input = map_user_rows(user_row, role_rows, permission_rows);
        let user = assemble_user(input)?;
        Ok(Some(user))
    }
}
