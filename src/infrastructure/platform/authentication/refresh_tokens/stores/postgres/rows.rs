//! PostgreSQL row definitions for refresh-token persistence.

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a row in `accounts.refresh_tokens`.
#[derive(Debug, FromRow)]
pub struct RefreshTokenRow {
    pub id: Uuid,
    pub family_id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub replaced_by_id: Option<Uuid>,
}
