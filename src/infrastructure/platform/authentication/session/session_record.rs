//! Redis-backed session payload.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::platform::identity::values::UserId;

/// Stored Redis session payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub user_id: String,
    pub created_at_unix: i64,
}

impl SessionRecord {
    /// Creates a new session record.
    pub fn new(user_id: &UserId, created_at_unix: i64) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            user_id: user_id.as_uuid().to_string(),
            created_at_unix,
        }
    }

    /// Returns the Redis key for this session.
    pub fn redis_key(&self) -> String {
        format!("auth:session:{}", self.session_id)
    }
}
