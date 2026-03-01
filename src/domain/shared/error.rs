//! Shared error contracts and traits.
//!
//! This module defines the standard interfaces that all domain errors must
//! implement to ensure consistent communication with external systems
//! (like frontends or API consumers) while supporting full localization.

use serde_json::Value;

/// A trait for domain errors to provide structured, localization-friendly payloads.
///
/// Instead of relying on hardcoded English strings, the backend returns a strictly
/// typed slug (`error_code`) and optional dynamic data (`context`). The frontend
/// uses the slug as a translation key and injects the context into the translated string.
pub trait ErrorCode {
    /// Returns the unique slug representing this specific error.
    ///
    /// # Example
    /// `"AUTH_EMAIL_MISSING_AT_SYMBOL"`
    fn error_code(&self) -> &'static str;

    /// Returns optional dynamic data required for frontend translations.
    ///
    /// For example, if a password policy fails, this might return:
    /// `{"min_length": 8, "actual_length": 5}`
    fn context(&self) -> Option<Value> {
        None
    }
}
