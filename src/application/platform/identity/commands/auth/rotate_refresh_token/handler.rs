//! Application handler for refresh-token rotation.

use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::{
    AccessTokenIssuer, NewRefreshTokenRecord, OpaqueTokenHasher, OpaqueTokenIssuer,
    RefreshTokenStore,
};
use crate::application::platform::identity::commands::auth::IssuedTokens;
use crate::application::shared::transaction::{Transaction, TransactionManager};
use crate::domain::platform::identity::ports::UserRepository;

use super::RefreshTokenCommand;

/// Rotates a valid refresh token into a new access-token / refresh-token pair.
pub struct RefreshTokenHandler<TM, UR, RTR> {
    tx_manager: TM,
    user_repo: Arc<UR>,
    access_token_issuer: Arc<dyn AccessTokenIssuer>,
    refresh_token_issuer: Arc<dyn OpaqueTokenIssuer>,
    refresh_token_hasher: Arc<dyn OpaqueTokenHasher>,
    refresh_token_store: Arc<RTR>,
    refresh_ttl_seconds: i64,
}

impl<TM, UR, RTR> RefreshTokenHandler<TM, UR, RTR> {
    pub fn new(
        tx_manager: TM,
        user_repo: Arc<UR>,
        access_token_issuer: Arc<dyn AccessTokenIssuer>,
        refresh_token_issuer: Arc<dyn OpaqueTokenIssuer>,
        refresh_token_hasher: Arc<dyn OpaqueTokenHasher>,
        refresh_token_store: Arc<RTR>,
        refresh_ttl_seconds: i64,
    ) -> Self {
        Self {
            tx_manager,
            user_repo,
            access_token_issuer,
            refresh_token_issuer,
            refresh_token_hasher,
            refresh_token_store,
            refresh_ttl_seconds,
        }
    }
}

impl<TM, UR, RTR> RefreshTokenHandler<TM, UR, RTR>
where
    TM: TransactionManager,
    for<'tx> UR: UserRepository<TM::Tx<'tx>>,
    for<'tx> RTR: RefreshTokenStore<TM::Tx<'tx>>,
{
    /// Rotates a refresh token and returns a new token pair.
    pub async fn handle(&self, command: RefreshTokenCommand) -> Result<IssuedTokens, AppError> {
        let token_hash = self
            .refresh_token_hasher
            .hash_token(&command.refresh_token)?;

        let mut tx = self.tx_manager.begin().await?;
        let now = Utc::now();

        let Some(existing) = self
            .refresh_token_store
            .find_by_token_hash(&mut tx, &token_hash)
            .await?
        else {
            tx.rollback().await?;
            return Err(AppError::Authentication("Invalid refresh token".into()));
        };

        if existing.revoked_at.is_some() {
            tracing::warn!(
                token_id = %existing.id,
                family_id = %existing.family_id,
                "Revoked refresh token was presented"
            );

            tx.rollback().await?;
            return Err(AppError::Authentication("Invalid refresh token".into()));
        }

        if existing.used_at.is_some() {
            tracing::warn!(
                token_id = %existing.id,
                family_id = %existing.family_id,
                "Refresh token reuse detected; revoking family"
            );

            self.refresh_token_store
                .revoke_family(&mut tx, existing.family_id, now)
                .await?;

            tx.commit().await?;
            return Err(AppError::Authentication("Invalid refresh token".into()));
        }

        if existing.expires_at <= now {
            tracing::debug!(
                token_id = %existing.id,
                family_id = %existing.family_id,
                "Expired refresh token was presented"
            );

            tx.rollback().await?;
            return Err(AppError::Authentication("Invalid refresh token".into()));
        }

        let user = match self
            .user_repo
            .find_by_id(&mut tx, &existing.user_id)
            .await?
        {
            Some(user) => user,
            None => {
                tracing::warn!(
                    token_id = %existing.id,
                    family_id = %existing.family_id,
                    user_id = %existing.user_id.as_uuid(),
                    "Refresh token belongs to a user that no longer exists; revoking family"
                );

                self.refresh_token_store
                    .revoke_family(&mut tx, existing.family_id, now)
                    .await?;

                tx.commit().await?;
                return Err(AppError::Authentication("Invalid refresh token".into()));
            }
        };

        let access_token = self.access_token_issuer.issue_access_token(&user.id)?;
        let raw_refresh_token = self.refresh_token_issuer.issue_token()?;
        let refresh_token_hash = self.refresh_token_hasher.hash_token(&raw_refresh_token)?;
        let replacement_id = Uuid::new_v4();

        let new_record = NewRefreshTokenRecord {
            id: replacement_id,
            family_id: existing.family_id,
            user_id: user.id,
            token_hash: refresh_token_hash,
            created_at: now,
            expires_at: now + Duration::seconds(self.refresh_ttl_seconds),
        };

        self.refresh_token_store
            .insert(&mut tx, &new_record)
            .await?;

        self.refresh_token_store
            .mark_rotated(&mut tx, existing.id, replacement_id, now)
            .await?;

        tx.commit().await?;

        Ok(IssuedTokens {
            access_token: access_token.token,
            refresh_token: raw_refresh_token,
            expires_in: access_token.expires_in,
        })
    }
}
