//! Admin API DTOs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BeaconDeviceResponse {
    pub id: i64,
    pub device_key: String,
    pub display_name: String,
    pub kind: String,
    pub priority: i32,
    pub capabilities: Vec<crate::types::AgentCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CreateBeaconDeviceTokenAdminRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CreateBeaconDeviceTokenAdminResponse {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BeaconDeviceTokenResponse {
    pub id: i64,
    pub device_id: i64,
    pub name: String,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = Option<String>))]
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = Option<String>))]
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct RevokeResponse {
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ToggleResponse {
    pub changed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct AdminAuditLogFilterQuery {
    pub user_id: Option<i64>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<i64>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), param(value_type = Option<String>))]
    pub after: Option<DateTime<Utc>>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), param(value_type = Option<String>))]
    pub before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AdminAuditLogResponse {
    pub id: i64,
    pub user_id: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<i64>,
    pub entity_name: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AdminSessionResponse {
    pub id: i64,
    pub user_id: i64,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub expires_at: chrono::DateTime<chrono::Utc>,
    #[cfg_attr(
        all(debug_assertions, feature = "openapi"),
        schema(value_type = Option<String>)
    )]
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(value_type = String))]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[cfg_attr(
        all(debug_assertions, feature = "openapi"),
        schema(value_type = Option<String>)
    )]
    pub last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
    pub current: bool,
}

impl From<crate::entities::beacon_device::Model> for BeaconDeviceResponse {
    fn from(value: crate::entities::beacon_device::Model) -> Self {
        Self {
            id: value.id,
            device_key: value.device_key,
            display_name: value.display_name,
            kind: value.kind,
            priority: value.priority,
            capabilities: parse_device_capabilities(&value.capabilities_json),
        }
    }
}

fn parse_device_capabilities(value: &str) -> Vec<crate::types::AgentCapability> {
    serde_json::from_str(value).unwrap_or_default()
}

impl From<crate::entities::beacon_device_token::Model> for BeaconDeviceTokenResponse {
    fn from(value: crate::entities::beacon_device_token::Model) -> Self {
        Self {
            id: value.id,
            device_id: value.device_id,
            name: value.name,
            last_used_at: value.last_used_at,
            revoked_at: value.revoked_at,
            created_at: value.created_at,
        }
    }
}

impl From<aster_forge_db::audit_log::Model> for AdminAuditLogResponse {
    fn from(value: aster_forge_db::audit_log::Model) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            action: value.action,
            entity_type: value.entity_type,
            entity_id: value.entity_id,
            entity_name: value.entity_name,
            details: value.details,
            ip_address: value.ip_address,
            user_agent: value.user_agent,
            created_at: value.created_at,
        }
    }
}
