use crate::api::handlers::identity::register_user;
use crate::api::state::AppState;
use axum::{Router, routing::post};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(register_user::handler))
}
