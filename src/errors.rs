//! Product error boundary.
//!
//! Keep product-facing status codes, response envelopes, localization, and audit wording outside
//! Forge. This template only maps shared infrastructure errors into a small product error enum.

use crate::api::error_code::ApiErrorCode;

/// Product result type.
pub type Result<T> = std::result::Result<T, AppError>;

/// Public HTTP status category carried by product errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiErrorStatus {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    TooManyRequests,
    InternalServerError,
}

/// Authentication failure category used to map stable API error codes without matching messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthErrorKind {
    TokenMissing,
    TokenInvalid,
    CredentialsFailed,
    CsrfCookieMissing,
    CsrfHeaderMissing,
    CsrfTokenInvalid,
    RequestSourceUntrusted,
    RequestOriginUntrusted,
    RequestRefererUntrusted,
    RequestSourceMissing,
}

/// Product error type used by the generated service skeleton.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Database or shared persistence failure.
    #[error("database error: {0}")]
    Database(String),
    /// Runtime assembly or lifecycle failure.
    #[error("runtime error: {0}")]
    Runtime(String),
    /// Runtime configuration or config-sync failure.
    #[error("config error: {0}")]
    Config(String),
    /// Runtime data directory preparation failure.
    #[error("io error: {0}")]
    Io(String),
    /// Mail delivery or outbox processing failure.
    #[error("mail error: {0}")]
    Mail(String),
    /// Product-facing error with a stable API code.
    #[error("{message}")]
    Public {
        status: ApiErrorStatus,
        code: ApiErrorCode,
        message: String,
    },
    /// Authentication or authorization failure.
    #[error("auth error: {message}")]
    Auth {
        kind: AuthErrorKind,
        message: String,
    },
    /// Request validation failure.
    #[error("validation error: {0}")]
    Validation(String),
}

impl AppError {
    pub fn public(status: ApiErrorStatus, code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::Public {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn validation_with_code(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::public(ApiErrorStatus::BadRequest, code, message)
    }

    pub fn not_found_with_code(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::public(ApiErrorStatus::NotFound, code, message)
    }

    pub fn conflict_with_code(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::public(ApiErrorStatus::Conflict, code, message)
    }

    pub fn forbidden_with_code(code: ApiErrorCode, message: impl Into<String>) -> Self {
        Self::public(ApiErrorStatus::Forbidden, code, message)
    }

    pub fn auth(kind: AuthErrorKind, message: impl Into<String>) -> Self {
        Self::Auth {
            kind,
            message: message.into(),
        }
    }

    pub fn auth_token_missing(message: impl Into<String>) -> Self {
        Self::auth(AuthErrorKind::TokenMissing, message)
    }

    pub fn auth_token_invalid(message: impl Into<String>) -> Self {
        Self::auth(AuthErrorKind::TokenInvalid, message)
    }

    pub fn auth_credentials_failed(message: impl Into<String>) -> Self {
        Self::auth(AuthErrorKind::CredentialsFailed, message)
    }
}

impl From<aster_forge_db::DbError> for AppError {
    fn from(error: aster_forge_db::DbError) -> Self {
        Self::Database(error.to_string())
    }
}

impl From<sea_orm::DbErr> for AppError {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::Database(error.to_string())
    }
}

impl From<aster_forge_runtime::AsterRuntimeError> for AppError {
    fn from(error: aster_forge_runtime::AsterRuntimeError) -> Self {
        Self::Runtime(error.to_string())
    }
}

impl From<aster_forge_config::ConfigCoreError> for AppError {
    fn from(error: aster_forge_config::ConfigCoreError) -> Self {
        Self::Config(error.to_string())
    }
}

impl From<config::ConfigError> for AppError {
    fn from(error: config::ConfigError) -> Self {
        Self::Config(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<aster_forge_crypto::CryptoError> for AppError {
    fn from(error: aster_forge_crypto::CryptoError) -> Self {
        Self::auth_token_invalid(error.to_string())
    }
}

impl From<aster_forge_mail::MailDeliveryError> for AppError {
    fn from(error: aster_forge_mail::MailDeliveryError) -> Self {
        Self::Mail(error.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        Self::auth_token_invalid(error.to_string())
    }
}

impl From<aster_forge_api::ApiError> for AppError {
    fn from(error: aster_forge_api::ApiError) -> Self {
        Self::Validation(error.message().to_string())
    }
}
