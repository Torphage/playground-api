use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};

use crate::application::error::AppError;
use crate::config::JwtConfig;

use super::Claims;

/// Verifies and decodes JWTs.
#[derive(Clone)]
pub struct JwtVerifier {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtVerifier {
    pub fn new(config: &JwtConfig) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.set_issuer(&[config.issuer.as_str()]);
        validation.set_audience(&[config.audience.as_str()]);

        Self {
            decoding_key: DecodingKey::from_secret(config.secret.as_bytes()),
            validation,
        }
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| AppError::Authentication(format!("Invalid bearer token: {e}")))?;

        Ok(token_data.claims)
    }
}
