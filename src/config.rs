//! Application configuration management.
//!
//! This module parses layered configuration (Defaults -> .env file -> Environment Variables)
//! and deserializes them into strictly typed, domain-specific structs using `serde`.
//! It guarantees that the application only boots if all necessary configuration is present.

use config::ConfigError;
use serde::Deserialize;

// =========================================================================
// GLOBAL CONFIGURATION
// =========================================================================

/// The root configuration object containing all subsystem settings.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub auth: AuthConfig,
    pub cors: CorsConfig,
    pub log_level: String,
}

impl Config {
    /// Loads the configuration hierarchy.
    ///
    /// This method will panic if required fields (like database URLs) are missing
    /// or if type conversion fails, ensuring the app "fails fast" on boot.
    pub fn load() -> Result<Self, ConfigError> {
        // Automatically load variables from a `.env` file if it exists.
        // We ignore the Result because in production, we rely on actual Env Vars, not the file.
        dotenvy::dotenv().ok();

        let builder = config::Config::builder()
            // Application Defaults
            .set_default("server.host", "0.0.0.0")
            .expect("Failed to set default server.host")
            // ServerConfig.port
            .set_default("server.port", 3000)
            .expect("Failed to set default server.port")
            // DatabaseConfig.max_connections
            .set_default("database.max_connections", 50)
            .expect("Failed to set default database.max_connections")
            // CorsConfig.allowed_origins
            .set_default("cors.allowed_origins", vec!["http://localhost:5173"])
            .expect("Failed to set default cors.allowed_origins")
            .set_default("log_level", "info")
            .expect("Failed to set default log_level")

            // Environment Variable Overrides
            // Allows overriding nested fields.
            // Example: `APP_DATABASE__URL=postgres://...` sets `database.url`.
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
            );

        // Build and deserialize, panicking with a clear message if anything is missing.
        let config_obj = builder.build().expect("Failed to build configuration hierarchy");

        config_obj.try_deserialize()
    }
}

// =========================================================================
// SUBSYSTEM CONFIGURATIONS
// =========================================================================

/// Configuration for the HTTP delivery layer.
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl ServerConfig {
    /// Helper to format the binding address for the Axum listener.
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Configuration for the primary PostgreSQL data store.
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// The full connection string required by sqlx.
    pub url: String,
    /// Maximum number of concurrent connections in the pool.
    pub max_connections: u32,
}

/// Configuration for the Redis caching infrastructure.
#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    /// The connection string for the Redis client.
    pub url: String,
}

/// Configuration for security and authentication mechanisms.
#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    /// The cryptographic secret used for signing JWTs.
    pub secret: String,
}

/// Configuration for Cross-Origin Resource Sharing.
#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    /// List of exact origin strings allowed to communicate with the API.
    pub allowed_origins: Vec<String>,
}
