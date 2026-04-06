//! Concrete refresh-token generation and hashing.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use crate::application::error::AppError;
use crate::application::ports::RefreshTokenService;

/// Generates opaque refresh tokens and hashes them for storage.
#[derive(Clone, Default)]
pub struct DefaultRefreshTokenService;

impl DefaultRefreshTokenService {
    pub fn new() -> Self {
        Self
    }
}

impl RefreshTokenService for DefaultRefreshTokenService {
    fn generate_token(&self) -> Result<String, AppError> {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);

        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }

    fn hash_token(&self, token: &str) -> Result<String, AppError> {
        let digest = Sha256::digest(token.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(digest))
    }
}
