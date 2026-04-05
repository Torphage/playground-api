use serde::{Deserialize, Serialize};

/// The data payload embedded inside the JWT.
///
/// These fields follow RFC 7519 standard JWT claims, plus one custom claim
/// (`roles`) kept for compatibility with the current design.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer
    pub iss: String,

    /// Audience
    pub aud: String,

    /// Subject (the user's unique identifier)
    pub sub: String,

    /// Issued at
    pub iat: usize,

    /// Expiration time
    pub exp: usize,

    /// JWT ID
    pub jti: String,

    /// Custom claim: the user's assigned roles
    pub roles: Vec<String>,
}
