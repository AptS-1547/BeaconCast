//! Product audit service boundary.
//!
//! Keep product audit action enums, detail schemas, presentation, permissions, and API filters in
//! the product repository. Forge owns only the lifecycle component and shared audit log mechanics.

use actix_web::{HttpRequest, http::header};
use chrono::Utc;
use sea_orm::ConnectionTrait;

use crate::api::dto::admin::{AdminAuditLogFilterQuery, AdminAuditLogResponse};
use crate::errors::Result;

const DEFAULT_AUDIT_LOG_LIMIT: u64 = 50;
const MAX_AUDIT_LOG_LIMIT: u64 = 200;
const MAX_AUDIT_IP_ADDRESS_LEN: usize = 45;
const MAX_AUDIT_USER_AGENT_LEN: usize = 512;

#[derive(Clone, Debug)]
pub struct AuditContext {
    pub actor_user_id: i64,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditContext {
    pub fn from_request(req: &HttpRequest, actor_user_id: i64) -> Self {
        Self {
            actor_user_id,
            ip_address: req
                .peer_addr()
                .map(|addr| bounded_audit_value(&addr.ip().to_string(), MAX_AUDIT_IP_ADDRESS_LEN)),
            user_agent: req
                .headers()
                .get(header::USER_AGENT)
                .and_then(|value| value.to_str().ok())
                .map(|value| bounded_audit_value(value, MAX_AUDIT_USER_AGENT_LEN)),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AdminAuditLogInput<'a> {
    pub ctx: &'a AuditContext,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Option<i64>,
    pub entity_name: Option<&'a str>,
}

pub async fn record_admin_action(
    state: &crate::runtime::AppState,
    input: AdminAuditLogInput<'_>,
    details: Option<serde_json::Value>,
) -> Result<()> {
    aster_forge_db::create_audit_log_row(
        state.db_handles.writer(),
        aster_forge_db::AuditLogCreate {
            user_id: input.ctx.actor_user_id,
            action: input.action.to_string(),
            entity_type: input.entity_type.to_string(),
            entity_id: input.entity_id,
            entity_name: input.entity_name.map(ToOwned::to_owned),
            details: details.map(|value| value.to_string()),
            ip_address: input.ctx.ip_address.clone(),
            user_agent: input.ctx.user_agent.clone(),
            created_at: Utc::now(),
        },
    )
    .await?;
    Ok(())
}

pub async fn record_admin_action_best_effort(
    state: &crate::runtime::AppState,
    input: AdminAuditLogInput<'_>,
    details: Option<serde_json::Value>,
) {
    if let Err(error) = record_admin_action(state, input, details).await {
        tracing::warn!(
            %error,
            action = input.action,
            entity_type = input.entity_type,
            entity_id = input.entity_id,
            "failed to record admin audit log"
        );
    }
}

fn bounded_audit_value(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }

    let mut end = max_len;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

pub async fn list_admin_audit_logs(
    state: &crate::runtime::AppState,
    page: aster_forge_api::LimitQuery,
    cursor: aster_forge_api::CreatedAtCursorQuery,
    filter: AdminAuditLogFilterQuery,
) -> Result<aster_forge_api::CursorPage<AdminAuditLogResponse, aster_forge_api::DateTimeIdCursor>> {
    let limit = page.limit_or(DEFAULT_AUDIT_LOG_LIMIT, MAX_AUDIT_LOG_LIMIT);
    let cursor = aster_forge_api::parse_datetime_id_cursor(
        cursor.after_created_at,
        cursor.after_id,
        "audit",
    )?;
    let action = filter
        .action
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let entity_type = filter
        .entity_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let slice = aster_forge_db::find_audit_logs_with_filters_cursor(
        state.db_handles.reader(),
        aster_forge_db::AuditLogQuery {
            user_id: filter.user_id,
            action,
            entity_type,
            entity_id: filter.entity_id,
            after: filter.after,
            before: filter.before,
            limit,
            cursor,
        },
    )
    .await?;
    let next_cursor = if slice.has_more {
        slice
            .items
            .last()
            .map(|item| aster_forge_api::DateTimeIdCursor {
                value: item.created_at,
                id: item.id,
            })
    } else {
        None
    };
    let items = slice
        .items
        .into_iter()
        .map(AdminAuditLogResponse::from)
        .collect();

    Ok(aster_forge_api::CursorPage::new(
        items,
        slice.total,
        limit,
        next_cursor,
    ))
}

pub async fn count_admin_audit_actions<C: ConnectionTrait>(
    db: &C,
    actions: &[&str],
) -> Result<u64> {
    Ok(
        aster_forge_db::count_audit_logs_created_between_with_actions(
            db,
            chrono::DateTime::<Utc>::UNIX_EPOCH,
            Utc::now() + chrono::Duration::seconds(1),
            actions,
        )
        .await?,
    )
}

pub mod runtime {
    //! Audit runtime component integration.

    /// Creates the audit runtime component used by the product entrypoint.
    pub fn audit_runtime_component() -> aster_forge_runtime::RuntimeComponentBundleRegistration<
        impl aster_forge_runtime::RuntimeComponentBundle,
    > {
        aster_forge_audit::audit_component(
            (),
            |()| async {
                tracing::info!("server start audit placeholder");
                Ok(())
            },
            |()| async {
                tracing::info!("server shutdown audit placeholder");
                Ok(())
            },
            |()| async {
                tracing::info!("audit manager flush placeholder");
                Ok(())
            },
        )
    }
}
