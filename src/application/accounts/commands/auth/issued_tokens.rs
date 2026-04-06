//! Shared application result for JWT issuance flows.

/// Pair of issued access/refresh tokens.
pub struct IssuedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}
