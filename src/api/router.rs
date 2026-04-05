use axum::Router;
use axum::http::{HeaderValue, Method};
use sentry_tower::NewSentryLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::api::handlers;
use crate::api::state::AppState;
use crate::config::CorsConfig;

pub fn create_router(state: AppState, cors_settings: CorsConfig) -> Router {
    // Construct the Layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        // Safely parse our strongly-typed Vec<String> into HeaderValues
        .allow_origin(
            cors_settings
                .allowed_origins
                .into_iter()
                .map(|origin| {
                    origin
                        .parse::<HeaderValue>()
                        .expect("Invalid CORS origin format")
                })
                .collect::<Vec<_>>(),
        )
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true);

    // Apply to Router
    Router::new()
        .nest("/", handlers::accounts::routes::routes())
        .with_state(state)
        .layer(cors)
        .layer(NewSentryLayer::new_from_top())
        .layer(TraceLayer::new_for_http())
}
