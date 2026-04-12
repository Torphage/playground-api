use axum::Router;
use axum::routing::post;

use super::{login, logout, refresh_token, register, token, token_revoke};
use crate::interfaces::http::axum::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(register::handler))
        .route("/login", post(login::handler))
        .route("/logout", post(logout::handler))
        .route("/token", post(token::handler))
        .route("/token/refresh", post(refresh_token::handler))
        .route("/token/revoke", post(token_revoke::handler))
}
