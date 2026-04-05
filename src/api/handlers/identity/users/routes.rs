use axum::Router;
use axum::routing::patch;

use crate::api::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}
