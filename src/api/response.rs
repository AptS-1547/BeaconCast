//! API response models.

use serde::{Deserialize, Serialize};

use crate::api::error_code::ApiErrorCode;

/// Optional machine-readable error details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ApiErrorInfo {
    pub code: ApiErrorCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
}

/// Empty success/error payload used where OpenAPI needs a concrete schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ApiEmptyData {}

/// Standard JSON response envelope for product APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub code: ApiErrorCode,
    pub msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorInfo>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn ok(data: T) -> Self {
        Self {
            code: ApiErrorCode::Success,
            msg: String::new(),
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: ApiErrorCode, msg: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            code,
            msg: msg.into(),
            data: None,
            error: Some(ApiErrorInfo {
                code,
                retryable: None,
            }),
        }
    }
}

/// Basic status response returned by the generated skeleton.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct StatusResponse {
    /// Cargo package name.
    pub service: &'static str,
    /// Public health or readiness status.
    pub status: &'static str,
}
