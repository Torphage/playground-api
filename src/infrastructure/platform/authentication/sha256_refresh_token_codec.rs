//! Concrete refresh-token issuing and hashing.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::{RefreshTokenHasher, RefreshTokenIssuer};

/// Issues opaque refresh tokens and hashes them for durable storage.
#[derive(Clone, Default)]
pub struct Sha256RefreshTokenCodec;

impl Sha256RefreshTokenCodec {
    /// Creates a new refresh-token codec.
    pub fn new() -> Self {
        Self
    }
}

impl RefreshTokenIssuer for Sha256RefreshTokenCodec {
    fn issue_refresh_token(&self) -> Result<String, AppError> {
        let mut bytes = [0u8; 32];
        OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|err| AppError::Infrastructure(err.to_string()))?;

        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }
}

impl RefreshTokenHasher for Sha256RefreshTokenCodec {
    fn hash_refresh_token(&self, token: &str) -> Result<String, AppError> {
        let digest = Sha256::digest(token.as_bytes());
        Ok(URL_SAFE_NO_PAD.encode(digest))
    }
}
