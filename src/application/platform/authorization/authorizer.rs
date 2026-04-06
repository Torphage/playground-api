//! Application-facing authorization abstraction.

use crate::application::error::AppError;
use crate::domain::platform::identity::values::Permission;

use super::principal::Principal;

/// Performs authorization checks against a principal.
pub trait Authorizer: Send + Sync {
    /// Returns true if the principal has the permission.
    fn has(&self, principal: &Principal, permission: &Permission) -> bool;

    /// Requires the permission or returns an authorization error.
    fn require(&self, principal: &Principal, permission: &Permission) -> Result<(), AppError> {
        if self.has(principal, permission) {
            Ok(())
        } else {
            Err(AppError::Authorization(format!(
                "Missing permission `{}`",
                permission.as_str()
            )))
        }
    }
}
