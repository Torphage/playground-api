use crate::api::handlers::identity_routes;
use crate::api::state::AppState;
use crate::config::CorsConfig;
use axum::Router;
use axum::http::{HeaderValue, Method};
use sentry_tower::NewSentryLayer;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

pub fn create_router(state: AppState, cors_settings: CorsConfig) -> Router {

    // 1. Construct the Layer
    let cors = CorsLayer::new()
        // A. ALLOWED METHODS
        // "GET, POST, etc." - Be explicit.
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])

        // B. ALLOWED ORIGINS
        // We iterate over the config list and convert Strings to HeaderValues
        .allow_origin(
            cors_settings.allowed_origins
                .iter()
                .map(|origin| origin.parse::<HeaderValue>().unwrap())
                .collect::<Vec<_>>()
        )

        // C. ALLOWED HEADERS
        // React sends 'Content-Type' (json) and 'Authorization' (bearer token).
        // If you don't allow these, the request fails.
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])

        // D. CREDENTIALS
        // If you use cookies or "withCredentials: true" in React, this must be true.
        // For JWTs in Headers, strictly speaking, you might not need it, but it's often safer to enable.
        .allow_credentials(true);

    // 2. Apply to Router
    Router::new()
        .nest("/users", identity::router()) // Merge User routes
        .with_state(state) // Inject State
        // 3. Middlewares (Order matters! Bottom runs first)
        .layer(cors)
        .layer(NewSentryLayer::new_from_top()) // Sentry
        .layer(TraceLayer::new_for_http())     // Logging
}
