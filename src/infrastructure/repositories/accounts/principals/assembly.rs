//! Backend-neutral assembly for the `Principal` authorization model.

use std::collections::HashSet;

use uuid::Uuid;

use crate::application::authorization::Principal;
use crate::application::error::AppError;
use crate::domain::accounts::values::{Permission, UserId};

/// Backend-neutral persistence input for assembling a `Principal`.
pub struct PrincipalAssemblyInput {
    pub user_id: Uuid,
    pub permission_slugs: Vec<String>,
}

/// Assembles a `Principal` from backend-neutral persistence data.
pub fn assemble_principal(input: PrincipalAssemblyInput) -> Result<Principal, AppError> {
    let permissions = input
        .permission_slugs
        .into_iter()
        .map(Permission::new)
        .collect::<HashSet<_>>();

    Ok(Principal::new(
        UserId::from_uuid(input.user_id),
        permissions,
    ))
}
