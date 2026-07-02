//! BeaconCast domain DTOs and policy enums.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ActivityInferenceSource {
    ServerRule,
    IdleDetector,
    Unknown,
}

impl ActivityInferenceSource {
    pub fn parse(value: &str) -> Self {
        match value {
            "server_rule" => Self::ServerRule,
            "idle_detector" => Self::IdleDetector,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ServerRule => "server_rule",
            Self::IdleDetector => "idle_detector",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum PublicMessagePart {
    Status,
    Activity,
    Project,
    Category,
    App,
    Source,
    BrowserContext,
    Context,
    GitBranch,
}

impl PublicMessagePart {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "status" => Some(Self::Status),
            "activity" => Some(Self::Activity),
            "project" => Some(Self::Project),
            "category" => Some(Self::Category),
            "app" => Some(Self::App),
            "source" => Some(Self::Source),
            "browser_context" => Some(Self::BrowserContext),
            "context" => Some(Self::Context),
            "git_branch" => Some(Self::GitBranch),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Activity => "activity",
            Self::Project => "project",
            Self::Category => "category",
            Self::App => "app",
            Self::Source => "source",
            Self::BrowserContext => "browser_context",
            Self::Context => "context",
            Self::GitBranch => "git_branch",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ProjectSnapshot {
    pub key: String,
    pub label: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PublicActivity {
    pub status: String,
    pub activity_kind: String,
    pub project: Option<ProjectSnapshot>,
    pub source: Option<String>,
    pub context_label: Option<String>,
    pub detail_badges: Vec<String>,
    pub message: PublicActivityMessage,
    pub updated_at: DateTime<Utc>,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PublicActivityMessage {
    pub headline: String,
    pub subline: Option<String>,
    pub badges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PublicProfile {
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct NowResponse {
    pub profile: PublicProfile,
    pub now: PublicActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ActivityLogEntry {
    pub public_id: String,
    pub activity: PublicActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ActivitySummaryResponse {
    pub window_days: u64,
    pub total_events: u64,
    pub latest: PublicActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentConfigResponse {
    pub poll_interval_seconds: u64,
    pub report_interval_seconds: u64,
    pub offline_after_seconds: u64,
    pub idle_after_seconds: u64,
    pub private_mode_enabled: bool,
    pub include_app_label: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum AgentCapability {
    ManualSignal,
    FrontmostApp,
}

impl AgentCapability {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ManualSignal => "manual_signal",
            Self::FrontmostApp => "frontmost_app",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentCapabilitiesRequest {
    pub capabilities: Vec<AgentCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentCapabilitiesResponse {
    pub accepted: bool,
    pub capabilities: Vec<AgentCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentPolicyResponse {
    pub config_poll_interval_seconds: u64,
    pub report_interval_seconds: u64,
    pub include_app_label: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ActivityMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_window_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
}

impl ActivityMetadata {
    pub fn is_empty(&self) -> bool {
        self.safe_window_context.is_none()
            && self.browser_context.is_none()
            && self.git_branch.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UpdateAgentPolicyRequest {
    pub config_poll_interval_seconds: u64,
    pub report_interval_seconds: u64,
    pub include_app_label: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentBeaconRequest {
    pub observed_at: Option<DateTime<Utc>>,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub source: String,
    pub idle: bool,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub metadata: ActivityMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BeaconAcceptedResponse {
    pub accepted: bool,
    pub event_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentUsageSpan {
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub started_at: DateTime<Utc>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub ended_at: DateTime<Utc>,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub source: String,
    pub idle: bool,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
    #[serde(default)]
    pub metadata: ActivityMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AgentUsageSpansRequest {
    pub spans: Vec<AgentUsageSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UsageSpansAcceptedResponse {
    pub accepted: bool,
    pub spans_accepted: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CreateBeaconDeviceRequest {
    pub device_key: String,
    pub display_name: String,
    #[serde(default = "default_device_kind")]
    pub kind: String,
    #[serde(default)]
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CreateBeaconDeviceResponse {
    pub id: i64,
    pub device_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CreateBeaconDeviceTokenRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CreateBeaconDeviceTokenResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AdminActivityEventResponse {
    pub id: i64,
    pub device_id: i64,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub observed_at: DateTime<Utc>,
    pub status: String,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub category: Option<String>,
    pub activity_kind: String,
    pub inference_source: ActivityInferenceSource,
    pub application_key: Option<String>,
    pub application_label: Option<String>,
    pub action_key: Option<String>,
    pub action_label: Option<String>,
    pub action_public_label: Option<String>,
    pub message_template: Option<String>,
    pub source: String,
    pub confidence: f32,
    pub metadata: ActivityMetadata,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AdminUsageSpanResponse {
    pub id: i64,
    pub device_id: i64,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub started_at: DateTime<Utc>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: u64,
    pub status: String,
    pub category: Option<String>,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub activity_kind: String,
    pub inference_source: ActivityInferenceSource,
    pub application_key: Option<String>,
    pub application_label: Option<String>,
    pub action_key: Option<String>,
    pub action_label: Option<String>,
    pub action_public_label: Option<String>,
    pub message_template: Option<String>,
    pub source: String,
    pub confidence: f32,
    pub metadata: ActivityMetadata,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UsageTotal {
    pub key: String,
    pub label: String,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AdminUsageSummaryResponse {
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub window_start: DateTime<Utc>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub window_end: DateTime<Utc>,
    pub total_seconds: u64,
    pub app_totals: Vec<UsageTotal>,
    pub project_totals: Vec<UsageTotal>,
    pub category_totals: Vec<UsageTotal>,
    pub status_totals: Vec<UsageTotal>,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::IntoParams))]
pub struct UsageSummaryQuery {
    #[serde(default = "default_usage_summary_days")]
    pub days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct VisibilityPolicyResponse {
    pub message_parts: Vec<PublicMessagePart>,
    pub public_history_enabled: bool,
    pub public_history_days: u64,
    pub private_mode_enabled: bool,
    pub private_mode_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UpdateVisibilityPolicyRequest {
    pub message_parts: Vec<PublicMessagePart>,
    pub public_history_enabled: bool,
    pub public_history_days: u64,
    pub private_mode_enabled: bool,
    pub private_mode_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ManualOverrideResponse {
    pub active: bool,
    pub id: Option<i64>,
    pub status: Option<String>,
    pub activity: Option<String>,
    #[cfg_attr(
        all(debug_assertions, feature = "openapi"),
        schema(value_type = Option<String>)
    )]
    pub starts_at: Option<DateTime<Utc>>,
    #[cfg_attr(
        all(debug_assertions, feature = "openapi"),
        schema(value_type = Option<String>)
    )]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct SetManualOverrideRequest {
    pub status: String,
    pub activity: String,
    #[serde(default)]
    pub expires_in_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ClearManualOverrideResponse {
    pub cleared: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ActivityActionResponse {
    pub id: i64,
    pub action_key: String,
    pub label: String,
    pub status: String,
    pub category: String,
    pub public_label: String,
    pub message_template: String,
    pub enabled: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UpsertActivityActionRequest {
    pub label: String,
    pub status: String,
    pub category: String,
    pub public_label: String,
    pub message_template: String,
    pub enabled: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ActivityApplicationResponse {
    pub id: i64,
    pub app_key: String,
    pub display_name: String,
    pub default_action_key: Option<String>,
    pub enabled: bool,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct UpsertActivityApplicationRequest {
    pub display_name: String,
    pub default_action_key: Option<String>,
    pub enabled: bool,
    pub aliases: Vec<String>,
}

fn default_confidence() -> f32 {
    1.0
}

fn default_device_kind() -> String {
    "desktop".to_string()
}

fn default_usage_summary_days() -> u64 {
    1
}
