use axum::Router;

use crate::interfaces::http::axum::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}
