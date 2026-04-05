//! HTTP handler for session logout.

use axum::{extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar};

use crate::api::error::ApiError;
use crate::api::state::AppState;
use crate::application::accounts::commands::auth::logout::{LogoutCommand, LogoutHandler};

pub async fn handler(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    let cookie_name = &state.config.authentication.session.cookie_name;

    let Some(cookie) = jar.get(cookie_name) else {
        return Ok((jar, StatusCode::NO_CONTENT));
    };

    let command = LogoutCommand {
        session_id: cookie.value().to_owned(),
    };

    let logout = LogoutHandler::new(state.sessions.store.clone());
    logout.handle(command).await?;

    let removal = Cookie::from(cookie_name.clone());
    let jar = jar.remove(removal);

    Ok((jar, StatusCode::NO_CONTENT))
}
