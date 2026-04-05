//! Application handler for password-based JWT issuance.

use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::{TokenGenerator, Transaction, TransactionManager};
use crate::domain::accounts::{
    AccountError,
    ports::{PasswordHasher, UserRepository},
    values::{Email, PlaintextPassword},
};

use super::IssueTokenCommand;

/// Handles the password-to-JWT use case.
pub struct IssueTokenHandler<TM, UR> {
    tx_manager: TM,
    user_repo: Arc<UR>,
    password_hasher: Arc<dyn PasswordHasher>,
    token_generator: Arc<dyn TokenGenerator>,
}

impl<TM, UR> IssueTokenHandler<TM, UR> {
    pub fn new(
        tx_manager: TM,
        user_repo: Arc<UR>,
        password_hasher: Arc<dyn PasswordHasher>,
        token_generator: Arc<dyn TokenGenerator>,
    ) -> Self {
        Self {
            tx_manager,
            user_repo,
            password_hasher,
            token_generator,
        }
    }
}

impl<TM, UR> IssueTokenHandler<TM, UR>
where
    TM: TransactionManager,
    for<'tx> UR: UserRepository<TM::Tx<'tx>>,
{
    /// Verifies credentials and issues a JWT access token.
    pub async fn handle(&self, command: IssueTokenCommand) -> Result<String, AppError> {
        let email = Email::try_from(command.email).map_err(AccountError::from)?;
        let password = PlaintextPassword::try_from(command.password).map_err(AccountError::from)?;

        let mut tx = self.tx_manager.begin().await?;

        let user = self
            .user_repo
            .find_by_email(&mut tx, &email)
            .await?
            .ok_or(AccountError::InvalidCredentials)?;

        let password_matches = self
            .password_hasher
            .verify(&password, &user.password_hash)
            .await
            .map_err(AppError::from)?;

        if !password_matches {
            tx.rollback().await?;
            return Err(AccountError::InvalidCredentials.into());
        }

        tx.commit().await?;

        self.token_generator.generate_token(&user)
    }
}
