use axum::Router;
use axum::routing::post;

use super::{login, logout, register, token};
use crate::api::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login::handler))
        .route("/logout", post(logout::handler))
        .route("/register", post(register::handler))
        .route("/token", post(token::handler))
}
