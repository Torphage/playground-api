//! Permission-based authorization adapter.

use crate::application::authorization::Authorizer;
use crate::application::authorization::Principal;
use crate::domain::accounts::values::Permission;

/// Simple permission presence checker.
#[derive(Debug, Clone, Default)]
pub struct PermissionAuthorizer;

impl PermissionAuthorizer {
    /// Constructs a new permission authorizer.
    pub fn new() -> Self {
        Self
    }
}

impl Authorizer for PermissionAuthorizer {
    fn has(&self, principal: &Principal, permission: &Permission) -> bool {
        principal.has(permission)
    }
}
