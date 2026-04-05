//! Application handler for password-based login.

use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::ports::{Transaction, TransactionManager};
use crate::domain::accounts::{
    AccountError,
    ports::{PasswordHasher, UserRepository},
    values::{Email, PlaintextPassword},
};
use crate::infrastructure::authentication::session::{FredSessionStore, SessionRecord};

use super::LoginCommand;

/// Handles the password login use case and creates a Redis-backed session.
pub struct LoginHandler<TM, UR> {
    tx_manager: TM,
    user_repo: Arc<UR>,
    password_hasher: Arc<dyn PasswordHasher>,
    session_store: Arc<FredSessionStore>,
}

impl<TM, UR> LoginHandler<TM, UR> {
    pub fn new(
        tx_manager: TM,
        user_repo: Arc<UR>,
        password_hasher: Arc<dyn PasswordHasher>,
        session_store: Arc<FredSessionStore>,
    ) -> Self {
        Self {
            tx_manager,
            user_repo,
            password_hasher,
            session_store,
        }
    }
}

impl<TM, UR> LoginHandler<TM, UR>
where
    TM: TransactionManager,
    for<'tx> UR: UserRepository<TM::Tx<'tx>>,
{
    /// Verifies credentials and creates a session.
    pub async fn handle(&self, command: LoginCommand) -> Result<SessionRecord, AppError> {
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

        let session = SessionRecord::new(&user.id, chrono::Utc::now().timestamp());
        self.session_store.create_session(&session).await?;

        Ok(session)
    }
}
