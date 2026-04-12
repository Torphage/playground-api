//! HTTP handler for password-based access-token issuance.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::application::platform::identity::commands::auth::issue_access_token::IssueTokenCommand;
use crate::interfaces::http::axum::error::ApiError;
use crate::interfaces::http::axum::state::AppState;

/// The expected JSON payload for token issuance.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub email: String,
    pub password: String,
}

/// The JSON response returned upon successful token issuance.
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
}

impl From<TokenRequest> for IssueTokenCommand {
    fn from(req: TokenRequest) -> Self {
        Self {
            email: req.email,
            password: req.password,
        }
    }
}

/// Handles the POST token request.
pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, ApiError> {
    let command = IssueTokenCommand::from(payload);

    let tokens = state
        .platform
        .handlers
        .auth
        .issue_access_token
        .handle(command)
        .await?;

    Ok(Json(TokenResponse {
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: "Bearer",
        expires_in: tokens.expires_in,
    }))
}
