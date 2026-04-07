//! HTTP handler for password-based login.

use axum::{Json, extract::State};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::platform::identity::commands::auth::login::LoginCommand;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub message: &'static str,
}

impl From<LoginRequest> for LoginCommand {
    fn from(req: LoginRequest) -> Self {
        Self {
            email: req.email,
            password: req.password,
        }
    }
}

pub async fn handler(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(payload): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), ApiError> {
    let command = LoginCommand::from(payload);

    let session = state.platform.handlers.auth.login.handle(command).await?;

    let cookie = Cookie::build((
        state.config.authentication.session.cookie_name.clone(),
        session.session_id,
    ))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Lax)
    .secure(state.config.authentication.session.secure_cookie) // set true in production over HTTPS
    .build();

    let jar = jar.add(cookie);

    Ok((
        jar,
        Json(LoginResponse {
            message: "Logged in successfully",
        }),
    ))
}
