//! Database row definitions for the User aggregate and RBAC system.
//!
//! This module defines the raw shapes of data as they exist in the PostgreSQL
//! database. These structs map exactly to the SQL tables and are heavily
//! coupled to the `sqlx` driver.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use std::fmt;
use uuid::Uuid;

// =========================================================================
// USER ROW
// =========================================================================

/// Represents a single record from the `identity.users` table.
///
/// This struct is purely a data transfer object (DTO) for the database.
/// It contains no business logic and uses raw primitive types.
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
    /// Custom debug formatter that redacts the password hash.
    ///
    /// This is a critical security measure. If an `sqlx` query fails and the
    /// application logs the surrounding context, this prevents the hashed
    /// password from leaking into Datadog, CloudWatch, or terminal outputs.
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

// =========================================================================
// RBAC ROWS
// =========================================================================

/// Represents a single record from the `identity.roles` table.
#[derive(Debug, FromRow)]
pub struct RoleRow {
    pub id: String,
    pub name: String,
}

/// Represents a single record from the `identity.permissions` table.
///
/// The description is optional in the database schema, hence the `Option<String>`.
#[derive(Debug, FromRow)]
pub struct PermissionRow {
    pub slug: String,
    pub description: Option<String>,
}

/// Represents the flattened result of a JOIN query fetching a user's permissions.
///
/// When we query a user's permissions, we often skip the intermediate tables
/// (`user_roles` and `role_permissions`) and select exactly the slugs the user
/// is entitled to. This struct captures that specific query output.
#[derive(Debug, FromRow)]
pub struct UserPermissionRow {
    pub permission_slug: String,
}
