//! HTTP handler for user registration.
//!
//! This module defines the entry point for the registration API. It handles
//! JSON extraction, maps the API DTO to an application command, and formats
//! the successful response.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::application::platform::identity::commands::auth::register_user::RegisterCommand;
use crate::interfaces::http::axum::error::ApiError;
use crate::interfaces::http::axum::state::AppState;

/// The expected JSON payload for a registration request.
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

/// The JSON response returned upon successful registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub message: &'static str,
}

impl From<RegisterRequest> for RegisterCommand {
    fn from(req: RegisterRequest) -> Self {
        Self {
            username: req.username,
            email: req.email,
            password: req.password,
        }
    }
}

/// Handles the POST registration request.
pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    let command = RegisterCommand::from(payload);

    let user_id = state
        .platform
        .handlers
        .auth
        .register_user
        .handle(command)
        .await?;

    Ok(Json(RegisterResponse {
        user_id: user_id.as_uuid(),
        message: "User registered successfully",
    }))
}
