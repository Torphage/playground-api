//! Application configuration management.
//!
//! This module handles loading configuration exclusively from standard
//! environment variables (and `.env` files for local development).
//! It guarantees that the application only boots if all necessary
//! configuration is present, failing fast with clear error messages.

use std::env;

// =========================================================================
// GLOBAL CONFIGURATION
// =========================================================================

/// The root configuration object containing all subsystem settings.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: Environment,
    pub startup: StartupConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub authentication: AuthenticationConfig,
    pub cors: CorsConfig,
    pub log_level: String,
}

impl AppConfig {
    /// Loads the configuration hierarchy from environment variables.
    ///
    /// This method will return an error if required fields (like database URLs)
    /// are missing or if type conversion fails, ensuring the app "fails fast" on boot.
    pub fn load() -> Result<Self, String> {
        // Automatically load variables from a `.env` file if it exists.
        // We ignore the Result because in production, we rely on actual Env Vars.
        dotenvy::dotenv().ok();

        // Helper to fetch required variables
        let get_env = |key: &str| -> Result<String, String> {
            env::var(key)
                .map_err(|_| format!("CRITICAL: Missing required environment variable: {key}"))
        };

        // Helper to fetch variables with a fallback default
        let get_env_or = |key: &str, default: &str| -> String {
            env::var(key).unwrap_or_else(|_| default.to_string())
        };

        let environment = match get_env_or("APP_ENV", "development").as_str() {
            "development" => Environment::Development,
            "test" => Environment::Test,
            "production" => Environment::Production,
            other => {
                return Err(format!(
                    "CRITICAL: APP_ENV must be one of development, test, production. Got: {other}"
                ));
            }
        };

        Ok(Self {
            environment,
            startup: StartupConfig {
                run_migrations: get_env_or("RUN_MIGRATIONS", "false")
                    .parse()
                    .unwrap_or(false),
            },
            server: ServerConfig {
                host: get_env_or("SERVER_HOST", "0.0.0.0"),
                port: get_env_or("SERVER_PORT", "3000")
                    .parse()
                    .map_err(|_| "CRITICAL: SERVER_PORT must be a valid u16".to_string())?,
            },
            database: DatabaseConfig {
                url: get_env("DATABASE_URL")?,
                max_connections: get_env_or("DATABASE_MAX_CONNECTIONS", "50")
                    .parse()
                    .map_err(|_| {
                        "CRITICAL: DATABASE_MAX_CONNECTIONS must be a valid u32".to_string()
                    })?,
            },
            redis: RedisConfig {
                url: get_env("REDIS_URL")?,
            },
            authentication: AuthenticationConfig {
                principal_cache_ttl_seconds: get_env_or("AUTH_PRINCIPAL_CACHE_TTL_SECONDS", "3600")
                    .parse()
                    .map_err(|_| {
                        "CRITICAL: AUTH_PRINCIPAL_CACHE_TTL_SECONDS must be a valid u64".to_string()
                    })?,
                jwt: JwtConfig {
                    secret: get_env("AUTH_JWT_SECRET")?,
                    issuer: get_env_or("AUTH_JWT_ISSUER", "my-app"),
                    audience: get_env_or("AUTH_JWT_AUDIENCE", "my-app-api"),
                    access_ttl_seconds: get_env_or("AUTH_JWT_ACCESS_TTL_SECONDS", "900")
                        .parse()
                        .map_err(|_| {
                            "CRITICAL: AUTH_JWT_ACCESS_TTL_SECONDS must be a valid i64".to_string()
                        })?,
                    refresh_ttl_seconds: get_env_or("AUTH_JWT_REFRESH_TTL_SECONDS", "2592000")
                        .parse()
                        .map_err(|_| {
                            "CRITICAL: AUTH_JWT_REFRESH_TTL_SECONDS must be a valid i64".to_string()
                        })?,
                },
                session: SessionConfig {
                    cookie_name: get_env_or("AUTH_SESSION_COOKIE_NAME", "sid"),
                    ttl_seconds: get_env_or("AUTH_SESSION_TTL_SECONDS", "604800")
                        .parse()
                        .map_err(|_| {
                            "CRITICAL: AUTH_SESSION_TTL_SECONDS must be a valid u64".to_string()
                        })?,
                    secure_cookie: get_env_or(
                        "AUTH_SESSION_SECURE_COOKIE",
                        &environment.is_production().to_string(),
                    )
                    .parse()
                    .map_err(|_| {
                        "CRITICAL: AUTH_SESSION_SECURE_COOKIE must be true or false".to_string()
                    })?,
                },
            },
            cors: CorsConfig {
                allowed_origins: get_env("CORS_ALLOWED_ORIGINS")?
                    .split(',')
                    // Remove accidental whitespace around the commas
                    .map(|s| s.trim().to_string())
                    // Ignore empty strings in case someone leaves a trailing comma
                    .filter(|s| !s.is_empty())
                    .collect(),
            },
            log_level: get_env_or("LOG_LEVEL", "info"),
        })
    }
}

// =========================================================================
// SUBSYSTEM CONFIGURATIONS
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Development,
    Test,
    Production,
}

impl Environment {
    pub fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}

/// Configuration for the HTTP delivery layer.
#[derive(Debug, Clone)]
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

/// Configuration for the application startup process.
#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub run_migrations: bool,
}

/// Configuration for the primary PostgreSQL data store.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// The full connection string required by sqlx.
    pub url: String,
    /// Maximum number of concurrent connections in the pool.
    pub max_connections: u32,
}

/// Configuration for the Redis caching infrastructure.
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// The connection string for the Redis client.
    pub url: String,
}

/// Configuration for security and authentication mechanisms.
#[derive(Debug, Clone)]
pub struct AuthenticationConfig {
    /// JWT-based authentication configuration.
    pub jwt: JwtConfig,

    /// Session-auth configuration.
    pub session: SessionConfig,

    /// Principal cache TTL in seconds.
    pub principal_cache_ttl_seconds: u64,
}

/// Configuration for JSON Web Tokens (JWTs).
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// The cryptographic secret used for signing JWTs.
    pub secret: String,

    /// The logical issuer of the JWTs produced by this API.
    pub issuer: String,

    /// The intended audience of the JWTs produced by this API.
    pub audience: String,

    /// Access-token lifetime in seconds.
    pub access_ttl_seconds: i64,

    /// Refresh-token lifetime in seconds.
    pub refresh_ttl_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Cookie name used for session authentication.
    pub cookie_name: String,

    /// Session lifetime in seconds.
    pub ttl_seconds: u64,

    /// Whether to use secure cookies (HTTPS-only).
    pub secure_cookie: bool,
}

/// Configuration for Cross-Origin Resource Sharing.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// List of exact origin strings allowed to communicate with the API.
    pub allowed_origins: Vec<String>,
}
