//! Data mapping and assembly for the User aggregate.
//!
//! This module translates raw `sqlx` database rows into strictly validated
//! domain aggregates. Because the `User` aggregate now includes RBAC
//! collections, this mapper acts as an assembler, combining data from
//! multiple SQL tables into a single cohesive domain entity.

use std::collections::HashSet;
use std::convert::TryFrom;

use crate::application::error::AppError;
use crate::domain::auth::entities::user::User;
use crate::domain::auth::values::email::Email;
use crate::domain::auth::values::password::PasswordHash;
use crate::domain::auth::values::user_id::UserId;
use crate::domain::auth::values::role::Role;
use crate::domain::auth::values::permission::Permission;

use super::rows::{RoleRow, UserPermissionRow, UserRow};

/// Assembles a complete `User` domain aggregate from its constituent database rows.
///
/// This function acts as the final boundary check between the database and the
/// pure business logic. If the database contains corrupted data (e.g., an invalid
/// email format), the assembly will fail.
///
/// # Arguments
/// * `user_row` - The core record from the `identity.users` table.
/// * `role_rows` - A collection of records representing the user's assigned roles.
/// * `permission_rows` - A flattened collection of permission slugs the user holds.
///
/// # Errors
/// Returns `AppError::Infrastructure` if data corruption is detected.
pub fn assemble_user(
    user_row: UserRow,
    role_rows: Vec<RoleRow>,
    permission_rows: Vec<UserPermissionRow>,
) -> Result<User, AppError> {
    // 1. Reconstruct simple value objects
    let id = UserId::from_uuid(user_row.id);
    let password_hash = PasswordHash::new(user_row.password_hash);

    // 2. Parse and validate the email
    // If this fails, our database contains a corrupted email string.
    let email = Email::try_from(user_row.email).map_err(|e| {
        tracing::error!("Data corruption: Invalid email in database for user {}: {:?}", user_row.id, e);
        AppError::Infrastructure("Database contains corrupted email data".into())
    })?;

    // 3. Map the raw role rows into `Role` value objects
    let roles: Vec<Role> = role_rows
        .into_iter()
        .map(|row| Role::new(row.id, row.name))
        .collect();

    // 4. Map the raw permission slugs into `Permission` value objects
    // We collect these directly into a HashSet for O(1) authorization checks in the domain.
    let permissions: HashSet<Permission> = permission_rows
        .into_iter()
        .map(|row| Permission::new(row.permission_slug))
        .collect();

    // 5. Reconstruct the aggregate root
    Ok(User::restore(
        id,
        email,
        password_hash,
        roles,
        permissions,
        user_row.created_at,
        user_row.updated_at,
    ))
}
