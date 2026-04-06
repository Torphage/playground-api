//! Application handler for session logout.

use std::sync::Arc;

use crate::application::error::AppError;
use crate::application::platform::identity::commands::auth::logout::LogoutCommand;
use crate::infrastructure::platform::authentication::session::FredSessionStore;

/// Handles session logout by deleting the Redis-backed session.
pub struct LogoutHandler {
    session_store: Arc<FredSessionStore>,
}

impl LogoutHandler {
    pub fn new(session_store: Arc<FredSessionStore>) -> Self {
        Self { session_store }
    }

    /// Deletes the current session.
    ///
    /// This operation is intentionally idempotent.
    pub async fn handle(&self, command: LogoutCommand) -> Result<(), AppError> {
        self.session_store.delete_session(&command.session_id).await
    }
}
