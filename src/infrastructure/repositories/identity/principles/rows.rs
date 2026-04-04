//! Row DTOs for principal loading queries.

/// Effective permission row for a principal query.
#[derive(Debug)]
pub struct PrincipalPermissionRow {
    pub permission_slug: String,
}
