//! PostgreSQL row definitions for principal loading.

use sqlx::FromRow;

/// Represents a flattened permission slug row for a principal query.
#[derive(Debug, FromRow)]
pub struct PrincipalPermissionRow {
    pub permission_slug: String,
}
