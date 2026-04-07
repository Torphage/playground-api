//! Refresh-token persistence contract.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::error::AppError;
use crate::domain::platform::identity::values::UserId;

/// Stored refresh-token lifecycle state.
#[derive(Debug, Clone)]
pub struct RefreshTokenRecord {
    pub id: Uuid,
    pub family_id: Uuid,
    pub user_id: UserId,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub replaced_by_id: Option<Uuid>,
}

/// Input required to persist a newly issued refresh token.
#[derive(Debug, Clone)]
pub struct NewRefreshTokenRecord {
    pub id: Uuid,
    pub family_id: Uuid,
    pub user_id: UserId,
    pub token_hash: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Persistence contract for refresh-token lifecycle management.
#[async_trait]
pub trait RefreshTokenStore<Tx>: Send + Sync {
    /// Inserts a newly issued refresh token.
    async fn insert(&self, tx: &mut Tx, new_token: &NewRefreshTokenRecord) -> Result<(), AppError>;

    /// Finds a refresh token by its stored hash.
    async fn find_by_token_hash(
        &self,
        tx: &mut Tx,
        token_hash: &str,
    ) -> Result<Option<RefreshTokenRecord>, AppError>;

    /// Marks a refresh token as rotated/used and links it to its replacement.
    async fn mark_rotated(
        &self,
        tx: &mut Tx,
        token_id: Uuid,
        replaced_by_id: Uuid,
        used_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// Revokes a single refresh token.
    async fn revoke_by_id(
        &self,
        tx: &mut Tx,
        token_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// Revokes every refresh token in a token family.
    async fn revoke_family(
        &self,
        tx: &mut Tx,
        family_id: Uuid,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), AppError>;

    /// Revokes every refresh token belonging to a user.
    async fn revoke_all_for_user(
        &self,
        tx: &mut Tx,
        user_id: &UserId,
        revoked_at: DateTime<Utc>,
    ) -> Result<(), AppError>;
}
