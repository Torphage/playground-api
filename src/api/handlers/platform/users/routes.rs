use axum::Router;

use crate::api::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
}
