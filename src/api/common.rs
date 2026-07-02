//! Shared API route helpers.

use actix_web::HttpResponse;

use crate::api::error_code::ApiErrorCode;
use crate::api::response::ApiResponse;
use crate::errors::{ApiErrorStatus, AppError, AuthErrorKind};

pub(super) async fn api_not_found() -> HttpResponse {
    HttpResponse::NotFound().json(ApiResponse::<()>::error(
        ApiErrorCode::EndpointNotFound,
        "endpoint not found",
    ))
}

pub(crate) fn app_error(error: AppError) -> HttpResponse {
    let (status, code, message) = match error {
        AppError::Auth { kind, message } => auth_error_response(kind, message),
        AppError::Public {
            status,
            code,
            message,
        } => (api_status_code(status), code, message),
        AppError::Validation(message) => (
            actix_web::http::StatusCode::BAD_REQUEST,
            ApiErrorCode::ValidationFailed,
            message,
        ),
        AppError::Database(_) => (
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::DatabaseError,
            "database operation failed".to_string(),
        ),
        AppError::Runtime(_) => (
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::RuntimeError,
            "runtime operation failed".to_string(),
        ),
        AppError::Config(_) => (
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::ConfigError,
            "configuration operation failed".to_string(),
        ),
        AppError::Io(_) => (
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::IoError,
            "io operation failed".to_string(),
        ),
        AppError::Mail(_) => (
            actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            ApiErrorCode::MailError,
            "mail operation failed".to_string(),
        ),
    };

    HttpResponse::build(status).json(ApiResponse::<()>::error(code, message))
}

fn api_status_code(status: ApiErrorStatus) -> actix_web::http::StatusCode {
    match status {
        ApiErrorStatus::BadRequest => actix_web::http::StatusCode::BAD_REQUEST,
        ApiErrorStatus::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
        ApiErrorStatus::Forbidden => actix_web::http::StatusCode::FORBIDDEN,
        ApiErrorStatus::NotFound => actix_web::http::StatusCode::NOT_FOUND,
        ApiErrorStatus::Conflict => actix_web::http::StatusCode::CONFLICT,
        ApiErrorStatus::TooManyRequests => actix_web::http::StatusCode::TOO_MANY_REQUESTS,
        ApiErrorStatus::InternalServerError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn auth_error_response(
    kind: AuthErrorKind,
    message: String,
) -> (actix_web::http::StatusCode, ApiErrorCode, String) {
    match kind {
        AuthErrorKind::TokenMissing => (
            actix_web::http::StatusCode::UNAUTHORIZED,
            ApiErrorCode::AuthTokenMissing,
            message,
        ),
        AuthErrorKind::TokenInvalid => (
            actix_web::http::StatusCode::UNAUTHORIZED,
            ApiErrorCode::AuthTokenInvalid,
            message,
        ),
        AuthErrorKind::CredentialsFailed => (
            actix_web::http::StatusCode::UNAUTHORIZED,
            ApiErrorCode::AuthCredentialsFailed,
            message,
        ),
        AuthErrorKind::CsrfCookieMissing => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthCsrfCookieMissing,
            message,
        ),
        AuthErrorKind::CsrfHeaderMissing => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthCsrfHeaderMissing,
            message,
        ),
        AuthErrorKind::CsrfTokenInvalid => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthCsrfTokenInvalid,
            message,
        ),
        AuthErrorKind::RequestSourceUntrusted => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthRequestSourceUntrusted,
            message,
        ),
        AuthErrorKind::RequestOriginUntrusted => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthRequestOriginUntrusted,
            message,
        ),
        AuthErrorKind::RequestRefererUntrusted => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthRequestRefererUntrusted,
            message,
        ),
        AuthErrorKind::RequestSourceMissing => (
            actix_web::http::StatusCode::FORBIDDEN,
            ApiErrorCode::AuthRequestSourceMissing,
            message,
        ),
    }
}
