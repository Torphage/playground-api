//! HTTP handler for user registration.
//!
//! This module defines the entry point for the registration API. It handles
//! JSON extraction, maps the API DTO to an Application Command, and
//! formats the successful response.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::identity::commands::register_user::RegisterUserCommand;

// =========================================================================
// DTOS (Data Transfer Objects)
// =========================================================================

/// The expected JSON payload for a registration request.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// The JSON response returned upon successful registration.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub message: &'static str,
}

// =========================================================================
// MAPPING LOGIC
// =========================================================================

impl From<RegisterRequest> for RegisterUserCommand {
    /// Maps the external API request DTO to the internal Application Command.
    ///
    /// This conversion happens entirely within the API layer, keeping the
    /// Application layer unaware of HTTP-specific structures.
    fn from(req: RegisterRequest) -> Self {
        Self {
            email: req.email,
            password: req.password,
        }
    }
}

// =========================================================================
// HANDLER
// =========================================================================

/// Handles the POST /auth/register request.
///
/// This function utilizes the `From` implementation to transform the payload
/// before dispatching the command to the orchestrated handler.
pub async fn handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    // Map API DTO to Application Command using our From impl
    let command = RegisterUserCommand::from(payload);

    // Dispatch the command
    // state.register_handler is wrapped in TransactionMiddleware, so this
    // call is automatically transactional.
    let user_id = command.execute(
        &state.pool,
        &state.repos.user,
        &state.crypto.password_hasher,
    ).await?;

    // 4. Return success
    Ok(Json(RegisterResponse {
        user_id: user_id.as_uuid(),
        message: "User registered successfully",
    }))
}
