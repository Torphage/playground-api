use axum::Router;

use crate::interfaces::http::axum::{
    platform::{auth, me, users},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes::routes())
        .nest("/me", me::routes::routes())
        .nest("/users", users::routes::routes())
}
