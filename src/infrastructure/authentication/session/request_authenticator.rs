//! Session-backed request authenticator.

use std::sync::Arc;

use async_trait::async_trait;
use axum::http::{header::COOKIE, request::Parts};
use uuid::Uuid;

use crate::api::authentication::AuthenticatedIdentity;
use crate::api::authentication::RequestAuthenticator;
use crate::application::error::AppError;
use crate::domain::accounts::values::UserId;

use super::fred_store::FredSessionStore;

/// Authenticates requests using a session cookie stored in Redis.
#[derive(Clone)]
pub struct SessionRequestAuthenticator {
    store: Arc<FredSessionStore>,
    cookie_name: String,
}

impl SessionRequestAuthenticator {
    /// Creates a new session request authenticator.
    pub fn new(store: Arc<FredSessionStore>, cookie_name: impl Into<String>) -> Self {
        Self {
            store,
            cookie_name: cookie_name.into(),
        }
    }
}

#[async_trait]
impl RequestAuthenticator for SessionRequestAuthenticator {
    async fn authenticate(&self, parts: &Parts) -> Result<Option<AuthenticatedIdentity>, AppError> {
        let Some(cookie_header) = parts
            .headers
            .get(COOKIE)
            .and_then(|value| value.to_str().ok())
        else {
            return Ok(None);
        };

        let Some(session_id) = extract_cookie(cookie_header, &self.cookie_name) else {
            return Ok(None);
        };

        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| AppError::Authentication("Invalid or expired session".into()))?;

        let user_uuid = Uuid::parse_str(&session.user_id).map_err(|e| {
            AppError::Infrastructure(format!("Stored session user_id is not a valid UUID: {e}"))
        })?;

        let user_id = UserId::from_uuid(user_uuid);

        Ok(Some(AuthenticatedIdentity::new(user_id)))
    }
}

/// Extracts a named cookie value from a raw Cookie header.
///
/// This is intentionally small and dependency-free for now.
fn extract_cookie<'a>(header: &'a str, cookie_name: &str) -> Option<&'a str> {
    header.split(';').map(str::trim).find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == cookie_name {
            Some(value)
        } else {
            None
        }
    })
}
