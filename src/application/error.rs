//! Global application error wrapper.
//!
//! This module defines the `AppError` type, which serves as the unified error
//! boundary for the entire application. It aggregates domain-specific business
//! errors and technical infrastructure failures into a single enum.

use serde_json::{Value, json};
use thiserror::Error;

use crate::domain::identity::IdentityError;
use crate::domain::shared::ErrorCode;

/// The global error type for the application workflow.
///
/// Handlers return `Result<T, AppError>`. This enum wraps specific domain
/// errors transparently while catching broader technical faults (like database
/// disconnects) without leaking sensitive infrastructure details to the frontend.
#[derive(Error, Debug)]
pub enum AppError {
    /// Wraps business rule violations from the Identity domain.
    /// The `#[from]` attribute allows using the `?` operator to automatically
    /// convert an `IdentityError` into an `AppError`.
    #[error(transparent)]
    Identity(#[from] IdentityError),

    /// Indicates that a requested resource (e.g., a specific user ID) does not exist.
    #[error("Resource not found: {0}")]
    NotFound(String),

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
            Self::Identity(err) => err.error_code(),

            // Standardize application-level failures
            Self::NotFound(_) => "SYS_NOT_FOUND",
            Self::Infrastructure(_) => "SYS_INFRASTRUCTURE_FAILURE",
            Self::Internal => "SYS_INTERNAL_ERROR",
        }
    }

    /// Resolves any dynamic translation context.
    fn context(&self) -> Option<Value> {
        match self {
            // Delegate domain-specific context
            Self::Identity(err) => err.context(),

            // Expose the name of the missing resource for better frontend messaging
            Self::NotFound(resource) => Some(json!({ "resource": resource })),

            // SECURITY: Never expose raw infrastructure errors or stack traces
            // in the context payload sent to the client.
            Self::Infrastructure(_) | Self::Internal => None,
        }
    }
}
