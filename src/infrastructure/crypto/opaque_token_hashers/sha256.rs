//! SHA-256-based opaque-token hasher.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::OpaqueTokenHasher;

/// Hashes opaque tokens using SHA-256.
#[derive(Clone, Default, Debug)]
pub struct Sha256OpaqueTokenHasher;

impl Sha256OpaqueTokenHasher {
    pub fn new() -> Self {
        Self
    }
}

impl OpaqueTokenHasher for Sha256OpaqueTokenHasher {
    fn hash_token(&self, token: &str) -> Result<String, AppError> {
        let digest = Sha256::digest(token.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(digest))
    }
}
