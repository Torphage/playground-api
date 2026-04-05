use crate::api::authentication::CurrentIdentity;
use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::accounts::commands::me::change_my_password::{Command, Handler};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use serde::Deserialize;

/// Request payload for this endpoint.
#[derive(Deserialize)]
pub struct ChangeMyPasswordRequest {
    pub new_password: String,
}

impl From<ChangeMyPasswordRequest> for Command {
    fn from(req: ChangeMyPasswordRequest) -> Self {
        Self {
            new_password: req.new_password,
        }
    }
}

/// Handles the request to change the current user's password.
pub async fn handler(
    State(state): State<AppState>,
    CurrentIdentity(identity): CurrentIdentity,
    Json(payload): Json<ChangeMyPasswordRequest>,
) -> Result<StatusCode, ApiError> {
    let command = Command::from(payload);

    let handler = Handler::new(
        state.tx_manager.clone(),
        state.repos.user.clone(),
        state.repos.principal.clone(),
        state.crypto.password_hasher.clone(),
        state.authorization.authorizer.clone(),
    );

    handler.handle(&identity, command).await?;

    Ok(StatusCode::NO_CONTENT)
}
