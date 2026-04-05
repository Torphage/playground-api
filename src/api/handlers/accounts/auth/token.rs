//! HTTP handler for password-based JWT issuance.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::accounts::commands::auth::issue_token::{
    IssueTokenCommand, IssueTokenHandler,
};

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

    let issue_token = IssueTokenHandler::new(
        state.tx_manager.clone(),
        state.repos.user.clone(),
        state.crypto.password_hasher.clone(),
        state.token_issuance.token_generator.clone(),
    );

    let access_token = issue_token.handle(command).await?;

    Ok(Json(TokenResponse {
        access_token,
        token_type: "Bearer",
        expires_in: state.config.authentication.jwt.access_ttl_seconds,
    }))
}
