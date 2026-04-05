//! Application handler for the change-email command.

use std::sync::Arc;

use crate::api::authentication::AuthenticatedIdentity;
use crate::application::authorization::Authorizer;
use crate::application::error::AppError;
use crate::application::identity::commands::me::change_my_password::Command;
use crate::application::ports::{PrincipalLoader, Transaction, TransactionManager};
use crate::domain::identity::values::{Permission, PlaintextPassword};
use crate::domain::identity::{IdentityError, PasswordHasher, ports::UserRepository};

pub struct Handler<TM, UR, PL> {
    tx_manager: TM,
    user_repo: Arc<UR>,
    principal_loader: Arc<PL>,
    password_hasher: Arc<dyn PasswordHasher>,
    authorizer: Arc<dyn Authorizer>,
}

impl<TM, UR, PL> Handler<TM, UR, PL> {
    pub fn new(
        tx_manager: TM,
        user_repo: Arc<UR>,
        principal_loader: Arc<PL>,
        password_hasher: Arc<dyn PasswordHasher>,
        authorizer: Arc<dyn Authorizer>,
    ) -> Self {
        Self {
            tx_manager,
            user_repo,
            principal_loader,
            password_hasher,
            authorizer,
        }
    }
}

impl<TM, UR, PL> Handler<TM, UR, PL>
where
    TM: TransactionManager,
    for<'tx> UR: UserRepository<TM::Tx<'tx>>,
    for<'tx> PL: PrincipalLoader<TM::Tx<'tx>>,
{
    /// Executes the workflow inside a transaction.
    pub async fn handle(
        &self,
        identity: &AuthenticatedIdentity,
        command: Command,
    ) -> Result<(), AppError> {
        // Validation
        let new_password =
            PlaintextPassword::try_from(command.new_password).map_err(IdentityError::from)?;

        // Begin transaction
        let mut tx = self.tx_manager.begin().await?;

        // Authentication
        let principal = self
            .principal_loader
            .load(&mut tx, &identity.user_id)
            .await?
            .ok_or_else(|| {
                AppError::Authentication("Authenticated user no longer exists".into())
            })?;

        // Authorization
        self.authorizer
            .require(&principal, &Permission::identity_self_change_email())?;

        // Load user
        let mut user = self
            .user_repo
            .find_by_id(&mut tx, &identity.user_id)
            .await?
            .ok_or(IdentityError::AccountNotFound)?;

        // Check if the new password matches the current password
        if self
            .password_hasher
            .verify(&new_password, &user.password_hash)
            .await?
        {
            return Err(IdentityError::PasswordMatchesCurrent.into());
        }

        // Hash new password
        let new_password_hash = self.password_hasher.hash(&new_password).await?;

        // Update user
        user.password_hash = new_password_hash;
        user.updated_at = chrono::Utc::now();

        // Save user
        self.user_repo.save(&mut tx, &user).await?;
        tx.commit().await?;

        Ok(())
    }
}
