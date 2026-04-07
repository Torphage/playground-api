//! Application handler for revoking all JWT refresh tokens belonging to a user.

use std::sync::Arc;

use chrono::Utc;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::RefreshTokenStore;
use crate::application::shared::{Transaction, TransactionManager};

use super::RevokeUserTokensCommand;

/// Revokes every refresh token belonging to the target user.
pub struct RevokeUserTokensHandler<TM, RTR> {
    tx_manager: TM,
    refresh_token_store: Arc<RTR>,
}

impl<TM, RTR> RevokeUserTokensHandler<TM, RTR> {
    pub fn new(tx_manager: TM, refresh_token_repo: Arc<RTR>) -> Self {
        Self {
            tx_manager,
            refresh_token_store: refresh_token_repo,
        }
    }
}

impl<TM, RTR> RevokeUserTokensHandler<TM, RTR>
where
    TM: TransactionManager,
    for<'tx> RTR: RefreshTokenStore<TM::Tx<'tx>>,
{
    pub async fn handle(&self, command: RevokeUserTokensCommand) -> Result<(), AppError> {
        let mut tx = self.tx_manager.begin().await?;

        self.refresh_token_store
            .revoke_all_for_user(&mut tx, &command.user_id, Utc::now())
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
