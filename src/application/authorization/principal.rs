use crate::domain::identity::values::{Permission, UserId};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Principal {
    pub user_id: UserId,
    permissions: HashSet<Permission>,
}

impl Principal {
    pub fn new(user_id: UserId, permissions: HashSet<Permission>) -> Self {
        Self {
            user_id,
            permissions,
        }
    }

    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn permissions(&self) -> &HashSet<Permission> {
        &self.permissions
    }
}
