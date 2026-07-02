//! Stable public API error codes.

use serde::{Deserialize, Serialize};

macro_rules! define_api_error_codes {
    ($($variant:ident => $value:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub enum ApiErrorCode {
            $(
                #[serde(rename = $value)]
                $variant,
            )+
        }

        impl ApiErrorCode {
            pub const ALL: &'static [Self] = &[
                $(Self::$variant,)+
            ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value,)+
                }
            }
        }
    };
}

#[cfg(all(debug_assertions, feature = "openapi"))]
impl utoipa::PartialSchema for ApiErrorCode {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        utoipa::openapi::ObjectBuilder::new()
            .schema_type(utoipa::openapi::schema::Type::String)
            .enum_values(Some(Self::ALL.iter().map(|code| code.as_str())))
            .into()
    }
}

#[cfg(all(debug_assertions, feature = "openapi"))]
impl utoipa::ToSchema for ApiErrorCode {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ApiErrorCode")
    }
}

define_api_error_codes! {
    // response: success envelope.
    Success => "success",

    // common/runtime: generic HTTP and infrastructure failures.
    BadRequest => "bad_request",
    NotFound => "not_found",
    Conflict => "conflict",
    RateLimited => "rate_limited",
    ValidationFailed => "validation.failed",
    EndpointNotFound => "endpoint.not_found",
    InternalServerError => "internal_server_error",
    DatabaseError => "database.error",
    ConfigError => "config.error",
    RuntimeError => "runtime.error",
    IoError => "io.error",
    MailError => "mail.error",

    // request validation: stable parsing and shape failures.
    ValidationRequestHeaderValueInvalid => "validation.request_header_value_invalid",
    ValidationRequestHostInvalid => "validation.request_host_invalid",
    ValidationRequestOriginInvalid => "validation.request_origin_invalid",
    ValidationRequestRefererInvalid => "validation.request_referer_invalid",
    ValidationRequestSchemeInvalid => "validation.request_scheme_invalid",
    PaginationCursorInvalid => "pagination.cursor_invalid",
    PaginationLimitInvalid => "pagination.limit_invalid",

    // auth/session/csrf: browser admin and token security failures.
    AuthTokenMissing => "auth.token_missing",
    AuthTokenInvalid => "auth.token_invalid",
    AuthTokenExpired => "auth.token_expired",
    AuthSessionExpired => "auth.session_expired",
    AuthSessionRevoked => "auth.session_revoked",
    AuthCsrfCookieMissing => "auth.csrf_cookie_missing",
    AuthCsrfHeaderMissing => "auth.csrf_header_missing",
    AuthCsrfTokenInvalid => "auth.csrf_token_invalid",
    AuthRequestSourceUntrusted => "auth.request_source_untrusted",
    AuthRequestOriginUntrusted => "auth.request_origin_untrusted",
    AuthRequestRefererUntrusted => "auth.request_referer_untrusted",
    AuthRequestSourceMissing => "auth.request_source_missing",
    AuthCredentialsFailed => "auth.credentials_failed",
    AuthAdminRequired => "auth.admin_required",
    Forbidden => "forbidden",

    // admin account/session management.
    AdminSetupAlreadyInitialized => "admin.setup_already_initialized",
    AdminUsernameInvalid => "admin.username_invalid",
    AdminPasswordTooWeak => "admin.password_too_weak",
    AdminDisplayNameRequired => "admin.display_name_required",
    AdminSessionNotFound => "admin.session_not_found",

    // beacon devices and device tokens.
    BeaconDeviceNotFound => "beacon_device.not_found",
    BeaconDeviceAlreadyExists => "beacon_device.already_exists",
    BeaconDeviceDisabled => "beacon_device.disabled",
    BeaconDeviceKeyInvalid => "beacon_device.key_invalid",
    BeaconDeviceDisplayNameInvalid => "beacon_device.display_name_invalid",
    BeaconDeviceKindInvalid => "beacon_device.kind_invalid",
    BeaconDevicePriorityInvalid => "beacon_device.priority_invalid",
    BeaconDeviceTokenNotFound => "beacon_device_token.not_found",
    BeaconDeviceTokenInvalid => "beacon_device_token.invalid",
    BeaconDeviceTokenRevoked => "beacon_device_token.revoked",
    BeaconDeviceTokenNameInvalid => "beacon_device_token.name_invalid",

    // activity ingest, public activity, and privacy policy.
    ActivitySignalInvalid => "activity.signal_invalid",
    ActivityStatusInvalid => "activity.status_invalid",
    ActivityConfidenceInvalid => "activity.confidence_invalid",
    ActivityTimestampInvalid => "activity.timestamp_invalid",
    ActivityMetadataInvalid => "activity.metadata_invalid",
    ActivityMetadataTooLarge => "activity.metadata_too_large",
    ActivityProjectKeyInvalid => "activity.project_key_invalid",
    ActivityProjectLabelInvalid => "activity.project_label_invalid",
    ActivityCategoryInvalid => "activity.category_invalid",
    ActivitySourceInvalid => "activity.source_invalid",
    ActivityActionNotFound => "activity_action.not_found",
    ActivityUsageSpanInvalid => "activity_usage_span.invalid",
    ActivityUsageSpanRangeInvalid => "activity_usage_span.range_invalid",
    ActivityUsageSpanBatchTooLarge => "activity_usage_span.batch_too_large",
    VisibilityPolicyInvalid => "visibility_policy.invalid",
    VisibilityMessagePartsInvalid => "visibility_policy.message_parts_invalid",
    VisibilityHistoryWindowInvalid => "visibility_policy.history_window_invalid",
    ManualOverrideInvalid => "manual_override.invalid",
    ManualOverrideExpiryInvalid => "manual_override.expiry_invalid",

    // runtime product configuration.
    SystemConfigKeyUnknown => "system_config.key_unknown",
    SystemConfigValueInvalid => "system_config.value_invalid",
    SystemConfigCategoryInvalid => "system_config.category_invalid",

    // audit and observability surfaces.
    AuditLogUnavailable => "audit_log.unavailable",
}
