use serde::{Deserialize, Serialize};

/// The data payload embedded inside the JWT.
///
/// These fields follow the RFC 7519 standard for JSON Web Tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (the user's unique identifier)
    pub sub: String,
    /// Issued at
    pub iat: usize,
    /// Expiration time
    pub exp: usize,
    /// Custom claim: the user's assigned roles
    pub roles: Vec<String>,
}
