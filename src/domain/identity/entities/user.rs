//! User aggregate root definition.
//!
//! This module contains the primary entity for the authentication domain.
//! It composes strictly validated value objects and RBAC collections to guarantee
//! secure identity and authorization boundaries.

use chrono::{DateTime, Utc};
use std::collections::HashSet;

use crate::domain::identity::values::{Email, PasswordHash, Permission, Role, UserId, Username};

/// The central entity representing an authenticated individual in the system.
///
/// As an aggregate root, `User` acts as the transaction boundary for domain
/// operations. It encapsulates its state, including its access control profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// The globally unique identifier for this user.
    pub id: UserId,

    /// The validated username, used for display purposes.
    pub username: Username,

    /// The validated email address, used as the primary login credential.
    pub email: Email,

    /// The securely hashed password. The plaintext password is never held here.
    pub password_hash: PasswordHash,

    /// The timestamp of when the user account was initially created.
    pub created_at: DateTime<Utc>,

    /// The timestamp of the last modification to the user's core data.
    pub updated_at: DateTime<Utc>,

    /// The specific roles assigned to this user (e.g., "admin").
    pub roles: Vec<Role>,

    /// The flattened, deduplicated set of all permissions granted to this user
    /// through their assigned roles. We use a HashSet for O(1) permission checks.
    pub permissions: HashSet<Permission>,
}

impl User {
    /// Constructs a completely new user account.
    ///
    /// This is typically called during the registration flow after the
    /// plaintext password has been successfully validated and hashed.
    ///
    /// By default, a newly created user has no roles or permissions. These must
    /// be explicitly assigned later via a separate workflow.
    pub fn create(username: Username, email: Email, password_hash: PasswordHash) -> Self {
        let now = Utc::now();
        Self {
            id: UserId::new(),
            username,
            email,
            password_hash,
            created_at: now,
            updated_at: now,
            roles: Vec::new(),
            permissions: HashSet::new(),
        }
    }

    /// Reconstructs a user from persistence (e.g., a database query).
    ///
    /// Unlike `create`, this method accepts an existing ID and timestamps,
    /// bypassing the generation of new identifiers.
    ///
    /// The infrastructure layer is responsible for gathering the user's roles
    /// and flattening their permissions before calling this method.
    pub fn restore(
        id: UserId,
        username: Username,
        email: Email,
        password_hash: PasswordHash,
        roles: Vec<Role>,
        permissions: HashSet<Permission>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            username,
            email,
            password_hash,
            roles,
            permissions,
            created_at,
            updated_at,
        }
    }

    // =========================================================================
    // AUTHORIZATION BEHAVIORS
    // =========================================================================

    /// Checks if the user holds a specific permission slug.
    ///
    /// # Example
    /// ```rust
    /// if !user.has_permission("kitchen.recipe.create") {
    ///     return Err(AuthError::Unauthorized);
    /// }
    /// ```
    pub fn has_permission(&self, slug: &str) -> bool {
        // We temporarily wrap the string reference to check the HashSet
        // without needing to allocate a new String.
        self.permissions.contains(&Permission::new(slug))
    }

    /// Checks if the user holds a specific role ID.
    pub fn has_role(&self, role_id: &str) -> bool {
        self.roles.iter().any(|r| r.id == role_id)
    }

    /// Updates the user's email address and refreshes the updated timestamp.
    pub fn change_email(&mut self, new_email: Email) {
        self.email = new_email;
        self.updated_at = Utc::now();
    }

    /// Updates the user's password hash and refreshes the updated timestamp.
    pub fn change_password(&mut self, new_hash: PasswordHash) {
        self.password_hash = new_hash;
        self.updated_at = Utc::now();
    }
}
