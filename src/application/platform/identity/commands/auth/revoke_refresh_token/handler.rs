//! Application handler for refresh-token-family revocation.

use std::sync::Arc;

use chrono::Utc;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::{RefreshTokenHasher, RefreshTokenStore};
use crate::application::shared::{Transaction, TransactionManager};

use super::RevokeTokenCommand;

/// Revokes the refresh-token family associated with the supplied refresh token.
pub struct RevokeTokenHandler<TM, RTR> {
    tx_manager: TM,
    refresh_token_hasher: Arc<dyn RefreshTokenHasher>,
    refresh_token_store: Arc<RTR>,
}

impl<TM, RTR> RevokeTokenHandler<TM, RTR> {
    pub fn new(
        tx_manager: TM,
        refresh_token_hasher: Arc<dyn RefreshTokenHasher>,
        refresh_token_store: Arc<RTR>,
    ) -> Self {
        Self {
            tx_manager,
            refresh_token_hasher,
            refresh_token_store,
        }
    }
}

impl<TM, RTR> RevokeTokenHandler<TM, RTR>
where
    TM: TransactionManager,
    for<'tx> RTR: RefreshTokenStore<TM::Tx<'tx>>,
{
    /// Revokes the refresh-token family tied to the supplied refresh token.
    ///
    /// This operation is intentionally idempotent.
    pub async fn handle(&self, command: RevokeTokenCommand) -> Result<(), AppError> {
        let token_hash = self
            .refresh_token_hasher
            .hash_refresh_token(&command.refresh_token)?;

        let mut tx = self.tx_manager.begin().await?;

        let Some(existing) = self
            .refresh_token_store
            .find_by_token_hash(&mut tx, &token_hash)
            .await?
        else {
            tx.rollback().await?;
            return Ok(());
        };

        self.refresh_token_store
            .revoke_family(&mut tx, existing.family_id, Utc::now())
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
