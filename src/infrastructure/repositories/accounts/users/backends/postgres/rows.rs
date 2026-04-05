//! PostgreSQL row definitions for the User aggregate and RBAC system.
//!
//! These structs map raw SQL query output and are tightly coupled to SQLx and
//! the PostgreSQL schema.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::fmt;
use uuid::Uuid;

/// Represents a single record from the `identity.users` table.
#[derive(FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Debug for UserRow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserRow")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password_hash", &"***REDACTED***")
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

/// Represents a single record from the `identity.roles` table.
#[derive(Debug, FromRow)]
pub struct RoleRow {
    pub id: String,
    pub name: String,
}

/// Represents a single record from the `identity.permissions` table.
#[derive(Debug, FromRow)]
pub struct PermissionRow {
    pub slug: String,
    pub description: Option<String>,
}

/// Represents the flattened result of a JOIN query fetching a user's permissions.
#[derive(Debug, FromRow)]
pub struct UserPermissionRow {
    pub permission_slug: String,
}
