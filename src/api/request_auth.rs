//! Request authentication helpers.

use actix_web::{HttpRequest, http::header};

pub const ADMIN_SESSION_COOKIE: &str = "beacon_admin_session";

pub fn admin_session_cookie(req: &HttpRequest) -> Option<String> {
    req.cookie(ADMIN_SESSION_COOKIE)
        .map(|cookie| cookie.value().to_string())
        .filter(|value| !value.trim().is_empty())
}

pub fn bearer_token(req: &HttpRequest) -> Option<&str> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.trim().is_empty())
}
