// api/state.rs
use sqlx::PgPool;
use std::sync::Arc;
use crate::domain::identity::ports::{
    UserRepository
};
use crate::config::Config; // If your handlers need access to config

#[derive(Clone)]
pub struct Repositories {
    pub user: Arc<dyn UserRepository>,
}

// A struct to hold your fully assembled state
#[derive(Clone)]
pub struct AppState {
    // Repositories (wrapped in Arc for cheap cloning across threads)
    pub repos: Repositories,

    // Database connections
    pub pool: PgPool,

    // Read-only configuration (e.g., JWT secrets)
    pub config: Arc<Config>,
}
