//! PostgreSQL implementation of the refresh-token repository port.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::{
    NewRefreshTokenRecord, RefreshTokenRecord, RefreshTokenStore,
};
use crate::domain::platform::identity::values::UserId;
use crate::infrastructure::db::postgres::PostgresTransaction;

use super::rows::RefreshTokenRow;

/// PostgreSQL-backed refresh-token repository.
#[derive(Default)]
pub struct PostgresRefreshTokenStore;

impl PostgresRefreshTokenStore {
    /// Creates a new repository instance.
    pub fn new() -> Self {
        Self
    }
}

fn assemble_refresh_token(row: RefreshTokenRow) -> RefreshTokenRecord {
    RefreshTokenRecord {
        id: row.id,
        family_id: row.family_id,
        user_id: UserId::from_uuid(row.user_id),
        token_hash: row.token_hash,
        created_at: row.created_at,
        expires_at: row.expires_at,
        used_at: row.used_at,
        revoked_at: row.revoked_at,
        replaced_by_id: row.replaced_by_id,
    }
}

#[async_trait]
impl<'a> RefreshTokenStore<PostgresTransaction<'a>> for PostgresRefreshTokenStore {
    async fn insert(
        &self,
        tx: &mut PostgresTransaction<'a>,
        new_token: &NewRefreshTokenRecord,
    ) -> Result<(), AppError> {
        let conn = tx.as_mut();

        sqlx::query!(
            r#"
            INSERT INTO identity.refresh_tokens (
                id,
                family_id,
                user_id,
                token_hash,
                created_at,
                expires_at,
                used_at,
                revoked_at,
                replaced_by_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, NULL, NULL, NULL)
            "#,
            new_token.id,
            new_token.family_id,
            new_token.user_id.as_uuid(),
            new_token.token_hash,
            new_token.created_at,
            new_token.expires_at,
        )
        .execute(&mut **conn)
        .await
        .map_err(|e| AppError::Infrastructure(format!("Failed to insert refresh token: {e}")))?;

        Ok(())
    }

    async fn find_by_token_hash(
        &self,
        tx: &mut PostgresTransaction<'a>,
        token_hash: &str,
    ) -> Result<Option<RefreshTokenRecord>, AppError> {
        let conn = tx.as_mut();

        let row = sqlx::query_as!(
            RefreshTokenRow,
            r#"
            SELECT
                id,
                family_id,
                user_id,
                token_hash,
                created_at,
                expires_at,
                used_at,
                revoked_at,
                replaced_by_id
            FROM identity.refresh_tokens
            WHERE token_hash = $1
            FOR UPDATE
            "#,
            token_hash
        )
        .fetch_optional(&mut **conn)
        .await
        .map_err(|e| {
            AppError::Infrastructure(format!("Failed to load refresh token by hash: {e}"))
        })?;

        Ok(row.map(assemble_refresh_token))
    }

    async fn mark_rotated(
        &self,
        tx: &mut PostgresTransaction<'a>,
        token_id: Uuid,
        replaced_by_id: Uuid,
        used_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let conn = tx.as_mut();

        let result = sqlx::query!(
            r#"
            UPDATE identity.refresh_tokens
            SET used_at = $2,
                replaced_by_id = $3
            WHERE id = $1
              AND used_at IS NULL
              AND revoked_at IS NULL
            "#,
            token_id,
            used_at,
            replaced_by_id
        )
        .execute(&mut **conn)
        .await
        .map_err(|e| {
            AppError::Infrastructure(format!("Failed to mark refresh token as rotated: {e}"))
        })?;

        if result.rows_affected() != 1 {
            return Err(AppError::Infrastructure(
                "Refresh token rotation affected an unexpected number of rows".into(),
            ));
        }

        Ok(())
    }

    async fn revoke_by_id(
        &self,
        tx: &mut PostgresTransaction<'a>,
        token_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let conn = tx.as_mut();

        sqlx::query!(
            r#"
            UPDATE identity.refresh_tokens
            SET revoked_at = $2
            WHERE id = $1
              AND revoked_at IS NULL
            "#,
            token_id,
            revoked_at
        )
        .execute(&mut **conn)
        .await
        .map_err(|e| AppError::Infrastructure(format!("Failed to revoke refresh token: {e}")))?;

        Ok(())
    }

    async fn revoke_family(
        &self,
        tx: &mut PostgresTransaction<'a>,
        family_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let conn = tx.as_mut();

        sqlx::query!(
            r#"
            UPDATE identity.refresh_tokens
            SET revoked_at = $2
            WHERE family_id = $1
              AND revoked_at IS NULL
            "#,
            family_id,
            revoked_at
        )
        .execute(&mut **conn)
        .await
        .map_err(|e| {
            AppError::Infrastructure(format!("Failed to revoke refresh token family: {e}"))
        })?;

        Ok(())
    }

    async fn revoke_all_for_user(
        &self,
        tx: &mut PostgresTransaction<'a>,
        user_id: &UserId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), AppError> {
        let conn = tx.as_mut();

        sqlx::query!(
            r#"
            UPDATE identity.refresh_tokens
            SET revoked_at = $2
            WHERE user_id = $1
              AND revoked_at IS NULL
            "#,
            user_id.as_uuid(),
            revoked_at
        )
        .execute(&mut **conn)
        .await
        .map_err(|e| {
            AppError::Infrastructure(format!("Failed to revoke all refresh tokens for user: {e}"))
        })?;

        Ok(())
    }
}
