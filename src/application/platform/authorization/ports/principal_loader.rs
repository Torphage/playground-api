//! Principal loading port.

use async_trait::async_trait;

use crate::application::error::AppError;
use crate::application::platform::authorization::Principal;
use crate::domain::platform::identity::values::UserId;

/// Loads a `Principal` inside an application workflow.
#[async_trait]
pub trait PrincipalLoader<Tx>: Send + Sync {
    async fn load(&self, tx: &mut Tx, user_id: &UserId) -> Result<Option<Principal>, AppError>;
}
