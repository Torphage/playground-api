use axum::Router;

use super::{auth, me, users};
use crate::api::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes::routes())
        .nest("/me", me::routes::routes())
        .nest("/users", users::routes::routes())
}
