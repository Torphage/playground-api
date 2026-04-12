//! Result of attempting authentication.

use crate::application::platform::authentication::AuthenticatedIdentity;

/// The result of attempting authentication.
#[derive(Debug, Clone)]
pub enum AuthenticationOutcome {
    /// Authentication succeeded and produced a caller identity.
    Authenticated(AuthenticatedIdentity),

    /// No supported authentication material was present for this authenticator.
    ///
    /// This is not an error. It allows composite authenticators or higher-level
    /// boundary adapters to decide whether authentication is optional or required.
    NotPresent,
}

impl AuthenticationOutcome {
    /// Returns the authenticated identity if authentication succeeded.
    pub fn into_identity(self) -> Option<AuthenticatedIdentity> {
        match self {
            Self::Authenticated(identity) => Some(identity),
            Self::NotPresent => None,
        }
    }
}
