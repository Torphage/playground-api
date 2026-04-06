//! HTTP handler for refresh-token revocation.

use axum::http::StatusCode;
use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::accounts::commands::auth::revoke_refresh_token::RevokeTokenCommand;

#[derive(Debug, Deserialize)]
pub struct RevokeTokenRequest {
    pub refresh_token: String,
}

impl From<RevokeTokenRequest> for RevokeTokenCommand {
    fn from(req: RevokeTokenRequest) -> Self {
        Self {
            refresh_token: req.refresh_token,
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<RevokeTokenRequest>,
) -> Result<StatusCode, ApiError> {
    let command = RevokeTokenCommand::from(payload);

    state
        .apps
        .accounts
        .auth
        .revoke_token
        .handle(command)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
