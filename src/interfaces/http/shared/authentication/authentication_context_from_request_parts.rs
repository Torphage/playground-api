//! HTTP boundary extraction for authentication context.

use http::{
    header::{AUTHORIZATION, COOKIE},
    request::Parts,
};

use crate::application::error::AppError;
use crate::application::platform::authentication::AuthenticationContext;

/// Builds an authentication context from HTTP request parts.
pub fn authentication_context_from_request_parts(
    parts: &Parts,
    session_cookie_name: &str,
) -> Result<AuthenticationContext, AppError> {
    let mut context = AuthenticationContext::empty();

    if let Some(token) = extract_bearer_token(parts)? {
        context = context.with_bearer_token(token);
    }

    if let Some(session_id) = extract_session_id(parts, session_cookie_name)? {
        context = context.with_session_id(session_id);
    }

    Ok(context)
}

/// Extracts a bearer token from the Authorization header.
///
/// Returning:
/// - `Ok(Some(token))` if a Bearer token is present
/// - `Ok(None)` if the header is absent
/// - `Err(...)` if the header is present but malformed or unsupported
fn extract_bearer_token(parts: &Parts) -> Result<Option<String>, AppError> {
    let Some(raw_header) = parts.headers.get(AUTHORIZATION) else {
        return Ok(None);
    };

    let header_value = raw_header
        .to_str()
        .map_err(|_| AppError::Authentication("Authorization header is not valid ASCII".into()))?;

    let mut segments = header_value.split_whitespace();

    let Some(scheme) = segments.next() else {
        return Err(AppError::Authentication(
            "Authorization header is empty".into(),
        ));
    };

    let Some(token) = segments.next() else {
        return Err(AppError::Authentication(
            "Bearer token is missing from Authorization header".into(),
        ));
    };

    if segments.next().is_some() {
        return Err(AppError::Authentication(
            "Authorization header must contain exactly two parts".into(),
        ));
    }

    if !scheme.eq_ignore_ascii_case("Bearer") {
        return Err(AppError::Authentication(format!(
            "Unsupported authorization scheme: {scheme}"
        )));
    }

    Ok(Some(token.to_owned()))
}

/// Extracts a session identifier from the Cookie header.
///
/// Returning:
/// - `Ok(Some(session_id))` if the configured session cookie is present
/// - `Ok(None)` if the Cookie header or named cookie is absent
/// - `Err(...)` if the Cookie header is malformed for text decoding
fn extract_session_id(parts: &Parts, cookie_name: &str) -> Result<Option<String>, AppError> {
    let Some(cookie_header) = parts.headers.get(COOKIE) else {
        return Ok(None);
    };

    let cookie_header = cookie_header
        .to_str()
        .map_err(|_| AppError::Authentication("Cookie header is not valid ASCII".into()))?;

    Ok(extract_cookie(cookie_header, cookie_name).map(ToOwned::to_owned))
}

/// Extracts a named cookie value from a raw Cookie header.
///
/// This is intentionally small and dependency-free for now.
fn extract_cookie<'a>(header: &'a str, cookie_name: &str) -> Option<&'a str> {
    header.split(';').map(str::trim).find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name == cookie_name {
            Some(value)
        } else {
            None
        }
    })
}
