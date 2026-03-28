//! User registration use case.
//!
//! This module defines the application command for registering a user and the
//! orchestration logic that validates input, enforces business rules, performs
//! password hashing, and persists the new aggregate inside a transaction.

use crate::application::error::AppError;
use crate::application::ports::{Transaction, TransactionManager};
use crate::domain::identity::entities::user::User;
use crate::domain::identity::error::IdentityError;
use crate::domain::identity::ports::PasswordHasher;
use crate::domain::identity::ports::UserRepository;
use crate::domain::identity::values::email::Email;
use crate::domain::identity::values::password::PlaintextPassword;
use crate::domain::identity::values::user_id::UserId;

/// The intent to register a new user in the system.
///
/// Commands contain raw, unvalidated primitive data. Validation is performed
/// during execution so that domain value objects are constructed inside the
/// use-case workflow.
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
}

impl RegisterUserCommand {
    /// Executes the registration workflow inside a transaction.
    ///
    /// # Workflow
    /// 1. Validate raw input into domain value objects.
    /// 2. Check email uniqueness.
    /// 3. Hash the password.
    /// 4. Create the `User` aggregate.
    /// 5. Persist and commit.
    pub async fn execute<TM, UR, PH>(
        self,
        tx_manager: &TM,
        user_repo: &UR,
        password_hasher: &PH,
    ) -> Result<UserId, AppError>
    where
        TM: TransactionManager,
        UR: UserRepository<TM::Tx>,
        PH: PasswordHasher + ?Sized,
    {
        let email = Email::try_from(self.email).map_err(IdentityError::from)?;
        let password = PlaintextPassword::try_from(self.password).map_err(IdentityError::from)?;

        let mut tx = tx_manager.begin().await?;

        if user_repo.find_by_email(&mut tx, &email).await?.is_some() {
            tx.rollback().await?;
            return Err(IdentityError::EmailAlreadyExists.into());
        }

        let password_hash = password_hasher.hash(&password).await?;
        let user = User::create(email, password_hash);
        let user_id = user.id;

        user_repo.save(&mut tx, &user).await?;
        tx.commit().await?;

        Ok(user_id)
    }
}
