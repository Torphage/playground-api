//! Global application error wrapper.
//!
//! This module defines the `AppError` type, which serves as the unified error
//! boundary for the entire application. It aggregates domain-specific business
//! errors and technical infrastructure failures into a single enum.

use serde_json::{Value, json};
use thiserror::Error;

use crate::domain::accounts::AccountError;
use crate::domain::shared::ErrorCode;

/// The global error type for the application workflow.
///
/// Handlers return `Result<T, AppError>`. This enum wraps specific domain
/// errors transparently while catching broader technical faults (like database
/// disconnects) without leaking sensitive infrastructure details to the frontend.
#[derive(Error, Debug)]
pub enum AppError {
    /// Wraps business rule violations from the Account domain.
    /// The `#[from]` attribute allows using the `?` operator to automatically
    /// convert an `AccountError` into an `AppError`.
    #[error(transparent)]
    Account(#[from] AccountError),

    /// Indicates a failure in authentication.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Indicates a failure in authorization.
    #[error("Authorization failed: {0}")]
    Authorization(String),

    /// Represents a failure in external infrastructure (database, cache, third-party API).
    /// The inner string contains the raw technical error for backend logging,
    /// but it will never be exposed to the user.
    #[error("Infrastructure failure: {0}")]
    Infrastructure(String),

    /// A catch-all for unexpected panic-level faults within the application layer.
    #[error("Internal system error")]
    Internal,
}

impl ErrorCode for AppError {
    /// Resolves the unified frontend localization slug.
    fn error_code(&self) -> &'static str {
        match self {
            // Transparently delegate to the specific domain error implementation
            Self::Account(err) => err.error_code(),

            // Standardize application-level failures
            Self::Authentication(_) => "SYS_AUTHENTICATION_FAILURE",
            Self::Authorization(_) => "SYS_AUTHORIZATION_FAILURE",
            Self::Infrastructure(_) => "SYS_INFRASTRUCTURE_FAILURE",
            Self::Internal => "SYS_INTERNAL_ERROR",
        }
    }

    /// Resolves any dynamic translation context.
    fn context(&self) -> Option<Value> {
        match self {
            // Delegate domain-specific context
            Self::Account(err) => err.context(),

            // Expose the authentication failure reason for better frontend messaging
            Self::Authentication(reason) => Some(json!({ "reason": reason })),

            // Expose the authorization failure reason for better frontend messaging
            Self::Authorization(reason) => Some(json!({ "reason": reason })),

            // SECURITY: Never expose raw infrastructure errors or stack traces
            // in the context payload sent to the client.
            Self::Infrastructure(_) | Self::Internal => None,
        }
    }
}
