//! Redis-backed session store using Fred.

use std::time::Duration;

use fred::prelude::*;

use crate::application::error::AppError;
use crate::infrastructure::db::redis::RedisClient;

use super::session_record::SessionRecord;

/// Redis-backed session store.
///
/// This is intentionally infrastructure-focused. It does not parse cookies or
/// know anything about Axum request extraction.
#[derive(Clone)]
pub struct FredSessionStore {
    client: RedisClient,
    ttl: Duration,
}

impl FredSessionStore {
    /// Creates a new session store with a fixed session TTL.
    pub fn new(client: RedisClient, ttl: Duration) -> Self {
        Self { client, ttl }
    }

    /// Persists a session record with a TTL.
    pub async fn create_session(&self, session: &SessionRecord) -> Result<(), AppError> {
        let ttl_secs = self.ttl.as_secs();

        let payload = serde_json::to_string(session).map_err(|_| AppError::Internal)?;

        let _: () = self
            .client
            .set(
                session.redis_key(),
                payload,
                Some(Expiration::EX(ttl_secs as i64)),
                None,
                false,
            )
            .await
            .map_err(|e| {
                AppError::Infrastructure(format!("Failed to create Redis session: {e}"))
            })?;

        Ok(())
    }

    /// Fetches a session by id.
    pub async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, AppError> {
        let key = format!("auth:session:{session_id}");

        let payload: Option<String> =
            self.client.get(key).await.map_err(|e| {
                AppError::Infrastructure(format!("Failed to fetch Redis session: {e}"))
            })?;

        let Some(payload) = payload else {
            return Ok(None);
        };

        let session = serde_json::from_str::<SessionRecord>(&payload).map_err(|e| {
            AppError::Infrastructure(format!("Failed to deserialize Redis session: {e}"))
        })?;

        Ok(Some(session))
    }

    /// Deletes a session by id.
    pub async fn delete_session(&self, session_id: &str) -> Result<(), AppError> {
        let key = format!("auth:session:{session_id}");

        let _: i64 = self.client.del(key).await.map_err(|e| {
            AppError::Infrastructure(format!("Failed to delete Redis session: {e}"))
        })?;

        Ok(())
    }
}
