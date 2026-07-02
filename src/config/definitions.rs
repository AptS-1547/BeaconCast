//! Runtime-adjustable BeaconCast configuration definitions.

use aster_forge_config::{
    ConfigDefinition, ConfigRegistry, ConfigValueLookup, ConfigValueType, ConfigVisibility,
};

pub const CONFIG_CATEGORY_VISIBILITY: &str = "beacon.visibility";
pub const CONFIG_CATEGORY_ACTIVITY: &str = "beacon.activity";
pub const CONFIG_CATEGORY_PRIVACY: &str = "beacon.privacy";
pub const CONFIG_CATEGORY_RETENTION: &str = "beacon.retention";
pub const CONFIG_CATEGORY_SECURITY: &str = "beacon.security";
pub const CONFIG_CATEGORY_AGENT: &str = "beacon.agent";

pub const SYSTEM_CONFIG_ALLOWED_CATEGORIES: &[&str] = &[
    CONFIG_CATEGORY_VISIBILITY,
    CONFIG_CATEGORY_ACTIVITY,
    CONFIG_CATEGORY_PRIVACY,
    CONFIG_CATEGORY_RETENTION,
    CONFIG_CATEGORY_SECURITY,
    CONFIG_CATEGORY_AGENT,
];

pub const VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY: &str = "visibility.public_message_parts";
pub const VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY: &str = "visibility.public_history_enabled";
pub const VISIBILITY_PUBLIC_HISTORY_DAYS_KEY: &str = "visibility.public_history_days";
pub const ACTIVITY_OFFLINE_AFTER_SECONDS_KEY: &str = "activity.offline_after_seconds";
pub const ACTIVITY_IDLE_AFTER_SECONDS_KEY: &str = "activity.idle_after_seconds";
pub const PRIVACY_PRIVATE_MODE_ENABLED_KEY: &str = "privacy.private_mode_enabled";
pub const PRIVACY_PRIVATE_MODE_LABEL_KEY: &str = "privacy.private_mode_label";
pub const PRIVACY_REDACT_UNKNOWN_PROJECTS_KEY: &str = "privacy.redact_unknown_projects";
pub const RETENTION_RAW_EVENT_DAYS_KEY: &str = "retention.raw_event_days";
pub const SECURITY_ADMIN_SESSION_TTL_SECONDS_KEY: &str = "security.admin_session_ttl_seconds";
pub const AGENT_CONFIG_POLL_INTERVAL_SECONDS_KEY: &str = "agent.config_poll_interval_seconds";
pub const AGENT_REPORT_INTERVAL_SECONDS_KEY: &str = "agent.report_interval_seconds";
pub const AGENT_INCLUDE_APP_LABEL_KEY: &str = "agent.include_app_label";

pub const DEPRECATED_SYSTEM_CONFIG_KEYS: &[&str] = &["visibility.default_level"];

fn default_public_message_parts() -> String {
    r#"["status","activity","project","context","browser_context","app","source","git_branch"]"#
        .to_string()
}

fn default_true() -> String {
    "true".to_string()
}

fn default_false() -> String {
    "false".to_string()
}

fn default_private_label() -> String {
    "Signal hidden".to_string()
}

fn default_public_history_days() -> String {
    "7".to_string()
}

fn default_offline_after_seconds() -> String {
    "1800".to_string()
}

fn default_idle_after_seconds() -> String {
    "600".to_string()
}

fn default_raw_event_days() -> String {
    "30".to_string()
}

fn default_admin_session_ttl_seconds() -> String {
    "604800".to_string()
}

fn default_agent_config_poll_interval_seconds() -> String {
    "300".to_string()
}

fn default_agent_report_interval_seconds() -> String {
    "30".to_string()
}

fn normalize_public_message_parts(
    _lookup: &dyn ConfigValueLookup,
    _key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    let parts = serde_json::from_str::<Vec<String>>(value).map_err(|_| {
        aster_forge_config::ConfigCoreError::invalid_value(
            "public message parts must be a JSON string array",
        )
    })?;
    let mut normalized = Vec::new();
    for part in parts {
        let part = part.trim();
        match part {
            "status" | "activity" | "project" | "category" | "app" | "browser_context"
            | "source" | "context" | "git_branch" => {
                if !normalized.iter().any(|existing| existing == part) {
                    normalized.push(part.to_string());
                }
            }
            _ => {
                return Err(aster_forge_config::ConfigCoreError::invalid_value(
                    "public message parts must contain only status, activity, project, category, app, source, browser_context, context, or git_branch",
                ));
            }
        }
    }
    serde_json::to_string(&normalized).map_err(Into::into)
}

fn normalize_bool(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    aster_forge_config::normalize_bool_config_value(key, value)
}

fn normalize_positive_u64(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    aster_forge_config::normalize_positive_u64_config_value(key, value)
}

pub const VISIBILITY_PUBLIC_MESSAGE_PARTS: ConfigDefinition = ConfigDefinition {
    key: VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
    label_i18n_key: "settings_visibility_public_message_parts_label",
    description_i18n_key: "settings_visibility_public_message_parts_desc",
    value_type: ConfigValueType::StringEnumSet,
    default_fn: default_public_message_parts,
    normalize_fn: Some(normalize_public_message_parts),
    category: CONFIG_CATEGORY_VISIBILITY,
    description: "Sanitized activity fields allowed in public activity display messages.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const VISIBILITY_PUBLIC_HISTORY_ENABLED: ConfigDefinition = ConfigDefinition {
    key: VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY,
    label_i18n_key: "settings_visibility_public_history_enabled_label",
    description_i18n_key: "settings_visibility_public_history_enabled_desc",
    value_type: ConfigValueType::Boolean,
    default_fn: default_true,
    normalize_fn: Some(normalize_bool),
    category: CONFIG_CATEGORY_VISIBILITY,
    description: "Whether public history endpoints may return sanitized historical sessions.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const VISIBILITY_PUBLIC_HISTORY_DAYS: ConfigDefinition = ConfigDefinition {
    key: VISIBILITY_PUBLIC_HISTORY_DAYS_KEY,
    label_i18n_key: "settings_visibility_public_history_days_label",
    description_i18n_key: "settings_visibility_public_history_days_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_public_history_days,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_VISIBILITY,
    description: "Maximum number of days exposed by public history endpoints.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const ACTIVITY_OFFLINE_AFTER_SECONDS: ConfigDefinition = ConfigDefinition {
    key: ACTIVITY_OFFLINE_AFTER_SECONDS_KEY,
    label_i18n_key: "settings_activity_offline_after_seconds_label",
    description_i18n_key: "settings_activity_offline_after_seconds_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_offline_after_seconds,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_ACTIVITY,
    description: "Seconds after which the current activity is treated as offline.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const ACTIVITY_IDLE_AFTER_SECONDS: ConfigDefinition = ConfigDefinition {
    key: ACTIVITY_IDLE_AFTER_SECONDS_KEY,
    label_i18n_key: "settings_activity_idle_after_seconds_label",
    description_i18n_key: "settings_activity_idle_after_seconds_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_idle_after_seconds,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_ACTIVITY,
    description: "Seconds after which an active device can be marked idle by policy.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const PRIVACY_PRIVATE_MODE_ENABLED: ConfigDefinition = ConfigDefinition {
    key: PRIVACY_PRIVATE_MODE_ENABLED_KEY,
    label_i18n_key: "settings_privacy_private_mode_enabled_label",
    description_i18n_key: "settings_privacy_private_mode_enabled_desc",
    value_type: ConfigValueType::Boolean,
    default_fn: default_false,
    normalize_fn: Some(normalize_bool),
    category: CONFIG_CATEGORY_PRIVACY,
    description: "Whether private mode overrides every public activity response.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const PRIVACY_PRIVATE_MODE_LABEL: ConfigDefinition = ConfigDefinition {
    key: PRIVACY_PRIVATE_MODE_LABEL_KEY,
    label_i18n_key: "settings_privacy_private_mode_label_label",
    description_i18n_key: "settings_privacy_private_mode_label_desc",
    value_type: ConfigValueType::String,
    default_fn: default_private_label,
    category: CONFIG_CATEGORY_PRIVACY,
    description: "Public label shown while private mode is active.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const PRIVACY_REDACT_UNKNOWN_PROJECTS: ConfigDefinition = ConfigDefinition {
    key: PRIVACY_REDACT_UNKNOWN_PROJECTS_KEY,
    label_i18n_key: "settings_privacy_redact_unknown_projects_label",
    description_i18n_key: "settings_privacy_redact_unknown_projects_desc",
    value_type: ConfigValueType::Boolean,
    default_fn: default_true,
    normalize_fn: Some(normalize_bool),
    category: CONFIG_CATEGORY_PRIVACY,
    description: "Whether unknown projects are hidden instead of shown by candidate label.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const RETENTION_RAW_EVENT_DAYS: ConfigDefinition = ConfigDefinition {
    key: RETENTION_RAW_EVENT_DAYS_KEY,
    label_i18n_key: "settings_retention_raw_event_days_label",
    description_i18n_key: "settings_retention_raw_event_days_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_raw_event_days,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_RETENTION,
    description: "Retention period for sanitized raw activity events.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const SECURITY_ADMIN_SESSION_TTL_SECONDS: ConfigDefinition = ConfigDefinition {
    key: SECURITY_ADMIN_SESSION_TTL_SECONDS_KEY,
    label_i18n_key: "settings_security_admin_session_ttl_seconds_label",
    description_i18n_key: "settings_security_admin_session_ttl_seconds_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_admin_session_ttl_seconds,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_SECURITY,
    description: "Lifetime for admin browser sessions.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const AGENT_CONFIG_POLL_INTERVAL_SECONDS: ConfigDefinition = ConfigDefinition {
    key: AGENT_CONFIG_POLL_INTERVAL_SECONDS_KEY,
    label_i18n_key: "settings_agent_config_poll_interval_seconds_label",
    description_i18n_key: "settings_agent_config_poll_interval_seconds_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_agent_config_poll_interval_seconds,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_AGENT,
    description: "Default interval for agents to refresh server-owned policy.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const AGENT_REPORT_INTERVAL_SECONDS: ConfigDefinition = ConfigDefinition {
    key: AGENT_REPORT_INTERVAL_SECONDS_KEY,
    label_i18n_key: "settings_agent_report_interval_seconds_label",
    description_i18n_key: "settings_agent_report_interval_seconds_desc",
    value_type: ConfigValueType::Number,
    default_fn: default_agent_report_interval_seconds,
    normalize_fn: Some(normalize_positive_u64),
    category: CONFIG_CATEGORY_AGENT,
    description: "Interval used by agents when publishing activity signals.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const AGENT_INCLUDE_APP_LABEL: ConfigDefinition = ConfigDefinition {
    key: AGENT_INCLUDE_APP_LABEL_KEY,
    label_i18n_key: "settings_agent_include_app_label_label",
    description_i18n_key: "settings_agent_include_app_label_desc",
    value_type: ConfigValueType::Boolean,
    default_fn: default_true,
    normalize_fn: Some(normalize_bool),
    category: CONFIG_CATEGORY_AGENT,
    description: "Whether agents may report a sanitized frontmost app label in activity signals.",
    visibility: ConfigVisibility::Authenticated,
    ..ConfigDefinition::private_system()
};

pub const ALL_CONFIGS: &[ConfigDefinition] = &[
    VISIBILITY_PUBLIC_MESSAGE_PARTS,
    VISIBILITY_PUBLIC_HISTORY_ENABLED,
    VISIBILITY_PUBLIC_HISTORY_DAYS,
    ACTIVITY_OFFLINE_AFTER_SECONDS,
    ACTIVITY_IDLE_AFTER_SECONDS,
    PRIVACY_PRIVATE_MODE_ENABLED,
    PRIVACY_PRIVATE_MODE_LABEL,
    PRIVACY_REDACT_UNKNOWN_PROJECTS,
    RETENTION_RAW_EVENT_DAYS,
    SECURITY_ADMIN_SESSION_TTL_SECONDS,
    AGENT_CONFIG_POLL_INTERVAL_SECONDS,
    AGENT_REPORT_INTERVAL_SECONDS,
    AGENT_INCLUDE_APP_LABEL,
];

pub static CONFIG_REGISTRY: ConfigRegistry = ConfigRegistry::new(ALL_CONFIGS);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        CONFIG_REGISTRY, SYSTEM_CONFIG_ALLOWED_CATEGORIES, VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
    };

    #[test]
    fn registry_has_valid_keys_and_categories() {
        CONFIG_REGISTRY.validate_unique_keys().unwrap();
        CONFIG_REGISTRY
            .validate_categories(SYSTEM_CONFIG_ALLOWED_CATEGORIES)
            .unwrap();
    }

    #[test]
    fn public_message_parts_normalizer_rejects_unknown_values() {
        let lookup = HashMap::new();

        assert!(
            CONFIG_REGISTRY
                .normalize_value(
                    &lookup,
                    VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
                    r#"["status","activity","project"]"#
                )
                .is_ok()
        );
        assert!(
            CONFIG_REGISTRY
                .normalize_value(
                    &lookup,
                    VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
                    r#"["window_title"]"#
                )
                .is_err()
        );
    }
}
