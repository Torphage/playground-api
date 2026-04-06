//! Application handler for refresh-token-family revocation.

use std::sync::Arc;

use chrono::Utc;

use crate::application::error::AppError;
use crate::application::ports::{
    RefreshTokenRepository, RefreshTokenService, Transaction, TransactionManager,
};

use super::RevokeTokenCommand;

/// Revokes the refresh-token family associated with the supplied refresh token.
pub struct RevokeTokenHandler<TM, RTR> {
    tx_manager: TM,
    refresh_token_repo: Arc<RTR>,
    refresh_token_service: Arc<dyn RefreshTokenService>,
}

impl<TM, RTR> RevokeTokenHandler<TM, RTR> {
    pub fn new(
        tx_manager: TM,
        refresh_token_repo: Arc<RTR>,
        refresh_token_service: Arc<dyn RefreshTokenService>,
    ) -> Self {
        Self {
            tx_manager,
            refresh_token_repo,
            refresh_token_service,
        }
    }
}

impl<TM, RTR> RevokeTokenHandler<TM, RTR>
where
    TM: TransactionManager,
    for<'tx> RTR: RefreshTokenRepository<TM::Tx<'tx>>,
{
    /// Revokes the refresh-token family tied to the supplied refresh token.
    ///
    /// This operation is intentionally idempotent.
    pub async fn handle(&self, command: RevokeTokenCommand) -> Result<(), AppError> {
        let token_hash = self
            .refresh_token_service
            .hash_token(&command.refresh_token)?;

        let mut tx = self.tx_manager.begin().await?;

        let Some(existing) = self
            .refresh_token_repo
            .find_by_token_hash(&mut tx, &token_hash)
            .await?
        else {
            tx.rollback().await?;
            return Ok(());
        };

        self.refresh_token_repo
            .revoke_family(&mut tx, existing.family_id, Utc::now())
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
