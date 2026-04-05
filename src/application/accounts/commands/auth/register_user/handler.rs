//! Application handler for the register-user command.

use std::sync::Arc;

use crate::application::accounts::commands::auth::register_user::RegisterCommand;
use crate::application::error::AppError;
use crate::application::ports::{Transaction, TransactionManager};
use crate::domain::accounts::{
    AccountError,
    entities::User,
    ports::{PasswordHasher, UserRepository},
    values::{Email, PlaintextPassword, UserId, Username},
};

/// Handles the register-user use case.
///
/// This handler owns its dependencies, which keeps the call-site small and
/// avoids lifetime-bearing handler types.
pub struct RegisterHandler<TM, UR> {
    tx_manager: TM,
    user_repo: Arc<UR>,
    password_hasher: Arc<dyn PasswordHasher>,
}

impl<TM, UR> RegisterHandler<TM, UR> {
    /// Creates a new handler instance.
    pub fn new(
        tx_manager: TM,
        user_repo: Arc<UR>,
        password_hasher: Arc<dyn PasswordHasher>,
    ) -> Self {
        Self {
            tx_manager,
            user_repo,
            password_hasher,
        }
    }
}

impl<TM, UR> RegisterHandler<TM, UR>
where
    TM: TransactionManager,
    for<'tx> UR: UserRepository<TM::Tx<'tx>>,
{
    /// Executes the registration workflow inside a transaction.
    pub async fn handle(&self, command: RegisterCommand) -> Result<UserId, AppError> {
        let username = Username::try_from(command.username).map_err(AccountError::from)?;
        let email = Email::try_from(command.email).map_err(AccountError::from)?;
        let password = PlaintextPassword::try_from(command.password).map_err(AccountError::from)?;

        let mut tx = self.tx_manager.begin().await?;

        if self
            .user_repo
            .find_by_email(&mut tx, &email)
            .await?
            .is_some()
        {
            tx.rollback().await?;
            return Err(AccountError::EmailAlreadyExists.into());
        }

        let password_hash = self.password_hasher.hash(&password).await?;
        let user = User::create(username, email, password_hash);
        let user_id = user.id;

        self.user_repo.save(&mut tx, &user).await?;
        tx.commit().await?;

        Ok(user_id)
    }
}
