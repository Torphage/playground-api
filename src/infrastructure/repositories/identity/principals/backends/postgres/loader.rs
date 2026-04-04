//! PostgreSQL implementation of the `PrincipalLoader` port.

use std::collections::HashSet;

use async_trait::async_trait;

use crate::application::authorization::Principal;
use crate::application::error::AppError;
use crate::application::ports::PrincipalLoader;
use crate::domain::identity::values::{Permission, UserId};
use crate::infrastructure::db::postgres::PostgresTransaction;

use super::mapper::map_permission_rows;
use super::rows::PrincipalPermissionRow;

/// PostgreSQL-backed principal loader.
#[derive(Default)]
pub struct PostgresPrincipalLoader;

impl PostgresPrincipalLoader {
    /// Creates a new repository instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl<'a> PrincipalLoader<PostgresTransaction<'a>> for PostgresPrincipalLoader {
    async fn load(
        &self,
        tx: &mut PostgresTransaction<'a>,
        user_id: &UserId,
    ) -> Result<Option<Principal>, AppError> {
        let user_uuid = user_id.as_uuid();
        let conn = tx.as_mut();

        let user_exists = sqlx::query!(
            r#"
            SELECT 1 AS "exists!"
            FROM identity.users
            WHERE id = $1
            "#,
            user_uuid
        )
        .fetch_optional(&mut **conn)
        .await
        .map_err(|e| {
            AppError::Infrastructure(format!("Failed to check principal existence: {e}"))
        })?;

        if user_exists.is_none() {
            return Ok(None);
        }

        let permission_rows: Vec<PrincipalPermissionRow> = sqlx::query_as!(
            PrincipalPermissionRow,
            r#"
            SELECT DISTINCT rp.permission_slug
            FROM identity.role_permissions rp
            JOIN identity.user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#,
            user_uuid
        )
        .fetch_all(&mut **conn)
        .await
        .map_err(|e| {
            AppError::Infrastructure(format!("Failed to fetch principal permissions: {e}"))
        })?;

        let input = map_permission_rows(user_id, permission_rows);
        let principal = assemble_principal(input)?;

        Ok(Some(principal))
    }
}
