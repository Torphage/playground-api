use axum::Router;
use axum::routing::patch;

use crate::api::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}
