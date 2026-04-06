//! HTTP handler for refresh-token rotation.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::accounts::commands::auth::rotate_refresh_token::RefreshTokenCommand;

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

impl From<RefreshTokenRequest> for RefreshTokenCommand {
    fn from(req: RefreshTokenRequest) -> Self {
        Self {
            refresh_token: req.refresh_token,
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, ApiError> {
    let command = RefreshTokenCommand::from(payload);

    let tokens = state
        .apps
        .accounts
        .auth
        .refresh_token
        .handle(command)
        .await?;

    Ok(Json(RefreshTokenResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: "Bearer",
        expires_in: tokens.expires_in,
    }))
}
