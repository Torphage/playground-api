use axum::Router;
use axum::routing::patch;

use super::register;
use crate::api::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/register", patch(register::handler))
}
