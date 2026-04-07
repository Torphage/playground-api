//! Redis/Fred client bootstrap.
//!
//! This module owns shared Redis client construction and initialization.
//! Higher-level components (sessions, caches, rate limiting, etc.) should
//! depend on this client rather than building their own.

use std::time::Duration;

use fred::prelude::*;

use crate::application::error::AppError;
use crate::config::RedisConfig;

/// Shared Redis client type used across the application.
///
/// Fred's client type is cheaply cloneable, so callers can store and pass
/// clones freely.
pub type RedisClient = Client;

/// Builds and initializes the shared Redis client.
pub async fn build_redis_client(config: &RedisConfig) -> Result<RedisClient, AppError> {
    let redis_config = Config::from_url(&config.url)
        .map_err(|e| AppError::Infrastructure(format!("Invalid Redis URL: {e}")))?;

    let client = Builder::from_config(redis_config)
        .with_connection_config(|connection| {
            connection.connection_timeout = Duration::from_secs(5);
            connection.max_command_attempts = 3;
        })
        .build()
        .map_err(|e| AppError::Infrastructure(format!("Failed to build Redis client: {e}")))?;

    client
        .init()
        .await
        .map_err(|e| AppError::Infrastructure(format!("Failed to initialize Redis client: {e}")))?;

    client.on_error(|(error, server)| async move {
        tracing::warn!(?server, error = %error, "Redis connection error");
        Ok(())
    });

    Ok(client)
}
