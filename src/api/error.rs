//! API-level error handling and HTTP response mapping.
//!
//! This module defines the `ApiError` wrapper, which is responsible for
//! translating application and domain errors into strict HTTP semantics.
//! It assigns appropriate HTTP status codes and formats the standardized
//! JSON error envelope expected by the frontend for localization.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use crate::application::error::AppError;
use crate::domain::identity::error::IdentityError;
use crate::domain::shared::error::ErrorCode;

/// A wrapper around `AppError` that implements Axum's `IntoResponse`.
///
/// Handlers in the API layer should return `Result<T, ApiError>`. This ensures
/// that the translation from internal application failures to external HTTP
/// responses happens in a single, centralized location.
#[derive(Debug)]
pub struct ApiError(pub AppError);

// Allows the `?` operator in handlers to automatically convert `AppError` into `ApiError`.
impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        Self(err)
    }
}

impl ApiError {
    /// Determines the appropriate HTTP status code for the underlying error.
    ///
    /// This method enforces the API's semantic correctness, mapping specific
    /// domain business rules or missing resources to their respective 4xx codes,
    /// and catching technical faults with 5xx codes.
    fn status_code(&self) -> StatusCode {
        match &self.0 {
            // Identity Context Mapping
            AppError::Identity(auth_err) => match auth_err {
                IdentityError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                IdentityError::AccountNotFound => StatusCode::NOT_FOUND,
                IdentityError::EmailAlreadyExists => StatusCode::CONFLICT,
                // Value object validations (like Email format) are inherently bad requests
                IdentityError::PasswordValidation(_) => StatusCode::BAD_REQUEST,
                IdentityError::EmailValidation(_) => StatusCode::BAD_REQUEST,
            },

            // Global Application Mapping
            AppError::NotFound(_) => StatusCode::NOT_FOUND,

            // Infrastructure and Internal mapping
            // These are strictly 500s. The user cannot fix these.
            AppError::Infrastructure(_) | AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    /// Consumes the `ApiError` to build the final HTTP response.
    ///
    /// This extracts the localization slug and context from the inner `AppError`
    /// using the `ErrorCode` trait, completely hiding internal system faults
    /// from external consumers.
    fn into_response(self) -> Response {
        let status = self.status_code();

        // Retrieve the localization slug and dynamic data via the ErrorCode trait.
        // self.0 accesses the inner AppError.
        let error_code_slug = self.0.error_code();
        let error_context = self.0.context();

        // For internal 500 errors, we explicitly log the underlying technical
        // failure to the server console, as the client will only see the generic slug.
        if status.is_server_error() {
            tracing::error!("Internal API Failure: {:?}", self.0);
        } else {
            // For 4xx errors, a debug log is usually sufficient.
            tracing::debug!("Client API Error: {:?}", self.0);
        }

        // Construct the standardized JSON envelope.
        let body = Json(json!({
            "error": {
                "code": error_code_slug,
                // Only include context if it exists, keeping payloads small.
                "context": error_context
            }
        }));

        (status, body).into_response()
    }
}
