//! Random opaque-token issuer.

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use rand::rngs::OsRng;

use crate::application::error::AppError;
use crate::application::platform::authentication::ports::OpaqueTokenIssuer;

/// Issues high-entropy opaque tokens using OS randomness.
#[derive(Clone, Debug)]
pub struct RandomOpaqueTokenIssuer {
    byte_length: usize,
}

impl Default for RandomOpaqueTokenIssuer {
    fn default() -> Self {
        Self::new()
    }
}

impl RandomOpaqueTokenIssuer {
    /// Creates an issuer with the default token length of 32 bytes.
    pub fn new() -> Self {
        Self { byte_length: 32 }
    }

    /// Creates an issuer with an explicit token length in bytes.
    pub fn with_byte_length(byte_length: usize) -> Self {
        assert!(
            byte_length > 0,
            "opaque token byte length must be greater than zero"
        );
        Self { byte_length }
    }

    /// Returns the configured token length in bytes.
    pub fn byte_length(&self) -> usize {
        self.byte_length
    }
}

impl OpaqueTokenIssuer for RandomOpaqueTokenIssuer {
    fn issue_token(&self) -> Result<String, AppError> {
        let mut bytes = vec![0u8; self.byte_length];
        OsRng.fill_bytes(&mut bytes);

        Ok(URL_SAFE_NO_PAD.encode(bytes))
    }
}
