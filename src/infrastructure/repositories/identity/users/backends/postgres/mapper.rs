//! PostgreSQL-specific mapping into backend-neutral user assembly input.

use crate::infrastructure::repositories::identity::users::assembly::{
    RoleAssemblyInput, UserAssemblyInput,
};

use super::rows::{RoleRow, UserPermissionRow, UserRow};

/// Maps PostgreSQL row DTOs into backend-neutral assembly input.
pub fn map_user_rows(
    user_row: UserRow,
    role_rows: Vec<RoleRow>,
    permission_rows: Vec<UserPermissionRow>,
) -> UserAssemblyInput {
    let roles = role_rows
        .into_iter()
        .map(|row| RoleAssemblyInput {
            id: row.id,
            name: row.name,
        })
        .collect();

    let permission_slugs = permission_rows
        .into_iter()
        .map(|row| row.permission_slug)
        .collect();

    UserAssemblyInput {
        id: user_row.id,
        username: user_row.username,
        email: user_row.email,
        password_hash: user_row.password_hash,
        roles,
        permission_slugs,
        created_at: user_row.created_at,
        updated_at: user_row.updated_at,
    }
}
