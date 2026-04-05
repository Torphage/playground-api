//! Redis-backed cache for principals.

use std::collections::HashSet;

use fred::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::authorization::Principal;
use crate::application::error::AppError;
use crate::domain::accounts::values::{Permission, UserId};
use crate::infrastructure::db::redis::RedisClient;

#[derive(Clone)]
pub struct RedisPrincipalCache {
    redis: RedisClient,
    ttl_seconds: u64,
}

impl RedisPrincipalCache {
    pub fn new(redis: RedisClient, ttl_seconds: u64) -> Self {
        Self { redis, ttl_seconds }
    }

    pub async fn get(&self, user_id: &UserId) -> Result<Option<Principal>, AppError> {
        let key = cache_key(user_id);

        let payload: Option<String> = self.redis.get(&key).await.map_err(|e| {
            AppError::Infrastructure(format!("Failed to fetch cached principal: {e}"))
        })?;

        let Some(payload) = payload else {
            return Ok(None);
        };

        let cached = serde_json::from_str::<RedisPrincipalEntry>(&payload).map_err(|e| {
            AppError::Infrastructure(format!("Failed to deserialize cached principal: {e}"))
        })?;

        cached.into_principal().map(Some)
    }

    pub async fn set(&self, principal: &Principal) -> Result<(), AppError> {
        let key = cache_key(&principal.user_id);
        let payload = serde_json::to_string(&RedisPrincipalEntry::from_principal(principal))
            .map_err(|_| AppError::Internal)?;

        let _: () = self
            .redis
            .set(
                key,
                payload,
                Some(Expiration::EX(self.ttl_seconds as i64)),
                None,
                false,
            )
            .await
            .map_err(|e| AppError::Infrastructure(format!("Failed to cache principal: {e}")))?;

        Ok(())
    }

    pub async fn delete(&self, user_id: &UserId) -> Result<(), AppError> {
        let key = cache_key(user_id);

        let _: i64 = self.redis.del(key).await.map_err(|e| {
            AppError::Infrastructure(format!("Failed to invalidate cached principal: {e}"))
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RedisPrincipalEntry {
    user_id: String,
    permissions: Vec<String>,
}

impl RedisPrincipalEntry {
    fn from_principal(principal: &Principal) -> Self {
        let permissions = principal
            .permissions()
            .iter()
            .map(|permission| permission.as_str().to_string())
            .collect();

        Self {
            user_id: principal.user_id.as_uuid().to_string(),
            permissions,
        }
    }

    fn into_principal(self) -> Result<Principal, AppError> {
        let user_uuid = Uuid::parse_str(&self.user_id).map_err(|e| {
            AppError::Infrastructure(format!("Cached principal user_id is not a valid UUID: {e}"))
        })?;

        let permissions = self
            .permissions
            .into_iter()
            .map(Permission::new)
            .collect::<HashSet<_>>();

        Ok(Principal::new(UserId::from_uuid(user_uuid), permissions))
    }
}

fn cache_key(user_id: &UserId) -> String {
    format!("identity:principal:{}", user_id.as_uuid())
}
