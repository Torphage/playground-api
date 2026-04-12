//! Transport-neutral authentication context.
//!
//! This type represents authentication material extracted from an incoming
//! boundary request, but does not depend on HTTP, Axum, headers, cookies,
//! or any specific delivery mechanism.
//!
//! Multiple authentication candidates may coexist at once. Higher-level
//! authenticators decide which mechanisms to try and in what order.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthenticationContext {
    bearer_token: Option<String>,
    session_id: Option<String>,
}

impl AuthenticationContext {
    /// Creates a new authentication context.
    pub fn new(bearer_token: Option<String>, session_id: Option<String>) -> Self {
        Self {
            bearer_token,
            session_id,
        }
    }

    /// Returns an empty authentication context.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Returns the bearer token, if present.
    pub fn bearer_token(&self) -> Option<&str> {
        self.bearer_token.as_deref()
    }

    /// Returns the session identifier, if present.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Returns true when no authentication material is present.
    pub fn is_empty(&self) -> bool {
        self.bearer_token.is_none() && self.session_id.is_none()
    }

    /// Sets the bearer token.
    pub fn with_bearer_token(mut self, bearer_token: impl Into<String>) -> Self {
        self.bearer_token = Some(bearer_token.into());
        self
    }

    /// Sets the session identifier.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }
}
