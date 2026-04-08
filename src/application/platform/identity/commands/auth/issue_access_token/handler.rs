//! Application handler for password-based access-token issuance.

use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::{
    AccessTokenIssuer, NewRefreshTokenRecord, OpaqueTokenHasher, OpaqueTokenIssuer,
    RefreshTokenStore,
};
use crate::application::platform::identity::commands::auth::IssuedTokens;
use crate::application::platform::identity::commands::auth::issue_access_token::IssueTokenCommand;
use crate::application::shared::{Transaction, TransactionManager};
use crate::domain::platform::identity::{
    AccountError,
    ports::{PasswordHasher, UserRepository},
    values::{Email, PlaintextPassword},
};

/// Handles the password-to-token use case.
pub struct IssueTokenHandler<TM, UR, RTR> {
    tx_manager: TM,
    user_repo: Arc<UR>,
    password_hasher: Arc<dyn PasswordHasher>,
    access_token_issuer: Arc<dyn AccessTokenIssuer>,
    refresh_token_issuer: Arc<dyn OpaqueTokenIssuer>,
    refresh_token_hasher: Arc<dyn OpaqueTokenHasher>,
    refresh_token_store: Arc<RTR>,
    refresh_ttl_seconds: i64,
}

impl<TM, UR, RTR> IssueTokenHandler<TM, UR, RTR> {
    pub fn new(
        tx_manager: TM,
        user_repo: Arc<UR>,
        password_hasher: Arc<dyn PasswordHasher>,
        access_token_issuer: Arc<dyn AccessTokenIssuer>,
        refresh_token_issuer: Arc<dyn OpaqueTokenIssuer>,
        refresh_token_hasher: Arc<dyn OpaqueTokenHasher>,
        refresh_token_store: Arc<RTR>,
        refresh_ttl_seconds: i64,
    ) -> Self {
        Self {
            tx_manager,
            user_repo,
            password_hasher,
            access_token_issuer,
            refresh_token_issuer,
            refresh_token_hasher,
            refresh_token_store,
            refresh_ttl_seconds,
        }
    }
}

impl<TM, UR, RTR> IssueTokenHandler<TM, UR, RTR>
where
    TM: TransactionManager,
    for<'tx> UR: UserRepository<TM::Tx<'tx>>,
    for<'tx> RTR: RefreshTokenStore<TM::Tx<'tx>>,
{
    /// Verifies credentials and issues an access token plus a refresh token.
    pub async fn handle(&self, command: IssueTokenCommand) -> Result<IssuedTokens, AppError> {
        let email = Email::try_from(command.email).map_err(AccountError::from)?;
        let password = PlaintextPassword::try_from(command.password).map_err(AccountError::from)?;

        let mut tx = self.tx_manager.begin().await?;

        let user = match self.user_repo.find_by_email(&mut tx, &email).await? {
            Some(user) => user,
            None => {
                tx.rollback().await?;
                return Err(AccountError::InvalidCredentials.into());
            }
        };

        let password_matches = self
            .password_hasher
            .verify(&password, &user.password_hash)
            .await
            .map_err(AppError::from)?;

        if !password_matches {
            tx.rollback().await?;
            return Err(AccountError::InvalidCredentials.into());
        }

        let now = Utc::now();
        let access_token = self.access_token_issuer.issue_access_token(&user.id)?;
        let raw_refresh_token = self.refresh_token_issuer.issue_token()?;
        let refresh_token_hash = self.refresh_token_hasher.hash_token(&raw_refresh_token)?;

        let refresh_record = NewRefreshTokenRecord {
            id: Uuid::new_v4(),
            family_id: Uuid::new_v4(),
            user_id: user.id,
            token_hash: refresh_token_hash,
            created_at: now,
            expires_at: now + Duration::seconds(self.refresh_ttl_seconds),
        };

        self.refresh_token_store
            .insert(&mut tx, &refresh_record)
            .await?;

        tx.commit().await?;

        Ok(IssuedTokens {
            access_token: access_token.token,
            refresh_token: raw_refresh_token,
            expires_in: access_token.expires_in,
        })
    }
}
