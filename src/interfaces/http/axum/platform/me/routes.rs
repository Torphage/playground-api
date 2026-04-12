use axum::Router;
use axum::routing::patch;

use super::change_my_password;
use crate::interfaces::http::axum::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/change-my-password", patch(change_my_password::handler))
}
