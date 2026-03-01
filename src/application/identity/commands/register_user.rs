//! User registration use case and command handler.
//!
//! This module orchestrates the workflow for registering a new user. It acts
//! as the bridge between the raw input of the Delivery layer (API) and the
//! strict rules of the Domain layer.

use crate::application::error::AppError;
use crate::domain::identity::entities::user::User;
use crate::domain::identity::error::IdentityError;
use crate::domain::identity::ports::PasswordHasher;
use crate::domain::identity::ports::UserRepository;
use crate::domain::identity::values::email::Email;
use crate::domain::identity::values::password::PlaintextPassword;
use crate::domain::identity::values::user_id::UserId;
// =========================================================================
// THE COMMAND (INPUT DTO)
// =========================================================================

/// The intent to register a new user in the system.
///
/// Commands contain raw, unvalidated data primitive types. Validation is
/// strictly deferred to the execution phase so that value objects are instantiated
/// securely within the transaction boundary.
pub struct RegisterUserCommand {
    pub email: String,
    pub password: String,
}

// =========================================================================
// THE HANDLER (ORCHESTRATOR)
// =========================================================================


impl RegisterUserCommand {
    /// Executes the registration workflow.
    ///
    /// # Workflow
    /// 1. Parse and validate the raw input into Domain Value Objects.
    /// 2. Enforce the business rule: Email must be unique.
    /// 3. Securely hash the validated plaintext password.
    /// 4. Instantiate the `User` aggregate root.
    /// 5. Persist the new aggregate via the Unit of Work.
    async fn execute(&self,
         _pool: &sqlx::PgPool,
         user_repo: &dyn UserRepository,
         password_hasher: &dyn PasswordHasher,
    ) -> Result<UserId, AppError> {

        // 1. Validation (The `?` operator maps DomainErrors to AppError automatically)
        let email = Email::try_from(self.email).map_err(IdentityError::from)?;
        let password = PlaintextPassword::try_from(self.password).map_err(IdentityError::from)?;

        // 2. Uniqueness Check
        // We query the repository using the Unit of Work to participate in the transaction.
        if user_repo.find_by_email(&email).await?.is_some() {
            return Err(IdentityError::EmailAlreadyExists.into());
        }

        // 3. Cryptography
        let password_hash = password_hasher.hash(&password).await?;

        // 4. Aggregate Instantiation
        let user = User::create(email, password_hash);
        let user_id = user.id;

        // 5. Persistence
        user_repo.save(&user).await?;

        Ok(user_id)
    }
}
