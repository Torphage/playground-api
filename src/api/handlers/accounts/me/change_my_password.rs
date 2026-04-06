use crate::api::authentication::CurrentIdentity;
use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::accounts::commands::me::change_my_password::Command;
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
    current_identity: CurrentIdentity,
    Json(payload): Json<ChangeMyPasswordRequest>,
) -> Result<StatusCode, ApiError> {
    let command = Command::from(payload);

    state
        .apps
        .accounts
        .me
        .change_my_password
        .handle(current_identity.identity(), command)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
