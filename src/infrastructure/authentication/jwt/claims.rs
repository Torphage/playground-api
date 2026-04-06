use serde::{Deserialize, Serialize};

/// The data payload embedded inside the JWT.
///
/// These are standard JWT claims used for validating and identifying the
/// authenticated caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// Subject (the user's unique identifier)
    pub sub: String,

    /// Issued at (Unix timestamp, seconds)
    pub iat: i64,

    /// Expiration time (Unix timestamp, seconds)
    pub exp: i64,

    /// JWT ID
    pub jti: String,
}
