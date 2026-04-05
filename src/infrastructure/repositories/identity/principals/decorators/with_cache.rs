//! Cache-backed implementation of the `PrincipalLoader` port.

use std::sync::Arc;

use async_trait::async_trait;

use crate::application::authorization::Principal;
use crate::application::error::AppError;
use crate::application::ports::PrincipalLoader;
use crate::domain::identity::values::UserId;
use crate::infrastructure::repositories::identity::principals::caches::redis::RedisPrincipalCache;

/// Decorates any `PrincipalLoader` with a Redis cache.
#[derive(Clone)]
pub struct CacheBackedPrincipalLoader<Inner> {
    inner: Arc<Inner>,
    cache: Arc<RedisPrincipalCache>,
}

impl<Inner> CacheBackedPrincipalLoader<Inner> {
    pub fn new(inner: Arc<Inner>, cache: Arc<RedisPrincipalCache>) -> Self {
        Self { inner, cache }
    }

    /// Invalidates the cached principal for a user.
    ///
    /// Call this after a successful transaction commit whenever roles or
    /// permissions may have changed.
    pub async fn invalidate(&self, user_id: &UserId) -> Result<(), AppError> {
        self.cache.delete(user_id).await
    }
}

#[async_trait]
impl<Tx, Inner> PrincipalLoader<Tx> for CacheBackedPrincipalLoader<Inner>
where
    Tx: Send,
    Inner: PrincipalLoader<Tx> + Send + Sync,
{
    async fn load(&self, tx: &mut Tx, user_id: &UserId) -> Result<Option<Principal>, AppError> {
        if let Some(principal) = self.cache.get(user_id).await? {
            return Ok(Some(principal));
        }

        let principal = self.inner.load(tx, user_id).await?;

        if let Some(ref principal) = principal {
            self.cache.set(principal).await?;
        }

        Ok(principal)
    }
}
