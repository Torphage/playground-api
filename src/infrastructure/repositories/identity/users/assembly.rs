//! Backend-neutral assembly for the `User` aggregate.
//!
//! This module reconstructs the domain aggregate from a backend-neutral
//! persistence shape. It should not know anything about SQLx, PostgreSQL,
//! MongoDB, BSON, or any other storage technology.

use std::collections::HashSet;
use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::domain::identity::{
    entities::User,
    values::{Email, PasswordHash, Permission, Role, UserId, Username},
};

/// Backend-neutral persistence input for assembling a `User`.
pub struct UserAssemblyInput {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<RoleAssemblyInput>,
    pub permission_slugs: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Backend-neutral persistence input for assembling a `Role`.
pub struct RoleAssemblyInput {
    pub id: String,
    pub name: String,
}

/// Assembles a complete `User` domain aggregate from backend-neutral persistence data.
///
/// This function acts as the final boundary check between storage and pure
/// business logic. If persisted data is corrupted (for example, an invalid
/// email string), assembly fails.
///
/// # Errors
/// Returns `AppError::Infrastructure` if persisted data is invalid.
pub fn assemble_user(input: UserAssemblyInput) -> Result<User, AppError> {
    let id = UserId::from_uuid(input.id);
    let password_hash = PasswordHash::new(input.password_hash);

    let username = Username::try_from(input.username).map_err(|e| {
        tracing::error!(
            "Data corruption: Invalid username in persisted data for user {}: {:?}",
            input.id,
            e
        );
        AppError::Infrastructure("Database contains corrupted username data".into())
    })?;

    let email = Email::try_from(input.email).map_err(|e| {
        tracing::error!(
            "Data corruption: Invalid email in persisted data for user {}: {:?}",
            input.id,
            e
        );
        AppError::Infrastructure("Database contains corrupted email data".into())
    })?;

    let roles: Vec<Role> = input
        .roles
        .into_iter()
        .map(|role| Role::new(role.id, role.name))
        .collect();

    let permissions: HashSet<Permission> = input
        .permission_slugs
        .into_iter()
        .map(Permission::new)
        .collect();

    Ok(User::restore(
        id,
        username,
        email,
        password_hash,
        roles,
        permissions,
        input.created_at,
        input.updated_at,
    ))
}
