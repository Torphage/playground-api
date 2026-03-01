use axum::{Router, routing::post};
use crate::api::handlers::identity::register_user;
use crate::api::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(register_user::handler))
}