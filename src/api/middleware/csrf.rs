//! Product boundary for CSRF validation.

use std::sync::OnceLock;

use actix_web::HttpRequest;
use aster_forge_actix_middleware::csrf::{
    CsrfError, CsrfErrorKind, CsrfTokenNames, RequestSourceMode,
};

use crate::api::error_code::ApiErrorCode;
use crate::errors::{AppError, AuthErrorKind, Result};

pub const ADMIN_CSRF_COOKIE: &str = "beacon_admin_csrf";
pub const ADMIN_CSRF_HEADER: &str = "X-Beacon-CSRF-Token";

static ADMIN_TOKEN_NAMES: OnceLock<CsrfTokenNames> = OnceLock::new();

pub fn admin_token_names() -> &'static CsrfTokenNames {
    ADMIN_TOKEN_NAMES.get_or_init(|| {
        CsrfTokenNames::new(ADMIN_CSRF_COOKIE, ADMIN_CSRF_HEADER).unwrap_or_else(|error| {
            tracing::error!(
                message = error.message(),
                "invalid built-in BeaconCast CSRF token names; falling back to Forge defaults"
            );
            CsrfTokenNames::default()
        })
    })
}

pub fn build_csrf_token() -> String {
    aster_forge_actix_middleware::csrf::build_csrf_token()
}

pub fn ensure_admin_cookie_write_allowed(req: &HttpRequest) -> Result<()> {
    if !aster_forge_actix_middleware::csrf::is_unsafe_method(req.method()) {
        return Ok(());
    }

    let public_site_origins = trusted_origins(req);
    aster_forge_actix_middleware::csrf::ensure_request_source_allowed(
        req,
        &public_site_origins,
        RequestSourceMode::OptionalWhenPresent,
    )
    .map_err(map_csrf_error)?;
    aster_forge_actix_middleware::csrf::ensure_double_submit_token_with_names(
        req,
        admin_token_names(),
    )
    .map_err(map_csrf_error)
}

fn trusted_origins(req: &HttpRequest) -> Vec<String> {
    let mut origins = request_origin(req)
        .map(|origin| vec![origin])
        .unwrap_or_default();
    let Some(state) = req.app_data::<actix_web::web::Data<crate::runtime::AppState>>() else {
        return origins;
    };
    for origin in &state.config.auth.trusted_origins {
        match aster_forge_utils::url::normalize_origin(origin, false) {
            Ok(normalized) if !origins.iter().any(|existing| existing == &normalized) => {
                origins.push(normalized);
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(
                    origin,
                    error = %error,
                    "ignoring invalid configured admin trusted origin"
                );
            }
        }
    }
    origins
}

fn request_origin(req: &HttpRequest) -> Option<String> {
    let conn = req.connection_info();
    aster_forge_utils::url::normalize_origin(&format!("{}://{}", conn.scheme(), conn.host()), false)
        .ok()
}

fn map_csrf_error(error: CsrfError) -> AppError {
    match error.kind() {
        CsrfErrorKind::TokenNameInvalid => AppError::Config(error.message().to_string()),
        CsrfErrorKind::CookieMissing => {
            AppError::auth(AuthErrorKind::CsrfCookieMissing, "missing CSRF cookie")
        }
        CsrfErrorKind::HeaderMissing => AppError::auth(
            AuthErrorKind::CsrfHeaderMissing,
            format!("missing {} header", admin_token_names().header_name_str()),
        ),
        CsrfErrorKind::TokenInvalid => {
            AppError::auth(AuthErrorKind::CsrfTokenInvalid, "invalid CSRF token")
        }
        CsrfErrorKind::RequestSchemeInvalid => AppError::validation_with_code(
            ApiErrorCode::ValidationRequestSchemeInvalid,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestHostInvalid => AppError::validation_with_code(
            ApiErrorCode::ValidationRequestHostInvalid,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestOriginInvalid => AppError::validation_with_code(
            ApiErrorCode::ValidationRequestOriginInvalid,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestRefererInvalid => AppError::validation_with_code(
            ApiErrorCode::ValidationRequestRefererInvalid,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestHeaderValueInvalid => AppError::validation_with_code(
            ApiErrorCode::ValidationRequestHeaderValueInvalid,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestSourceUntrusted => AppError::auth(
            AuthErrorKind::RequestSourceUntrusted,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestOriginUntrusted => AppError::auth(
            AuthErrorKind::RequestOriginUntrusted,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestRefererUntrusted => AppError::auth(
            AuthErrorKind::RequestRefererUntrusted,
            error.message().to_string(),
        ),
        CsrfErrorKind::RequestSourceMissing => AppError::auth(
            AuthErrorKind::RequestSourceMissing,
            error.message().to_string(),
        ),
    }
}
