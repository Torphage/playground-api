//! PostgreSQL-specific mapping into backend-neutral principal assembly input.

use crate::domain::platform::identity::values::UserId;
use crate::infrastructure::platform::authorization::principals::assembly::PrincipalAssemblyInput;

use super::rows::PrincipalPermissionRow;

/// Maps PostgreSQL permission rows into backend-neutral principal assembly input.
pub fn map_permission_rows(
    user_id: &UserId,
    permission_rows: Vec<PrincipalPermissionRow>,
) -> PrincipalAssemblyInput {
    let permission_slugs = permission_rows
        .into_iter()
        .map(|row| row.permission_slug)
        .collect();

    PrincipalAssemblyInput {
        user_id: user_id.as_uuid(),
        permission_slugs,
    }
}
