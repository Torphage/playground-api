//! BLAKE3-based opaque-token hasher.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::OpaqueTokenHasher;

/// Hashes opaque tokens using BLAKE3.
#[derive(Clone, Default, Debug)]
pub struct Blake3OpaqueTokenHasher;

impl Blake3OpaqueTokenHasher {
    pub fn new() -> Self {
        Self
    }
}

impl OpaqueTokenHasher for Blake3OpaqueTokenHasher {
    fn hash_token(&self, token: &str) -> Result<String, AppError> {
        let digest = blake3::hash(token.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(digest.as_bytes()))
    }
}
