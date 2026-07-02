//! Beacon device and event repository helpers.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, Set,
};

use crate::api::error_code::ApiErrorCode;
use crate::entities::{
    activity_action, activity_application, activity_application_alias, activity_event,
    activity_usage_span, beacon_device, beacon_device_token, manual_override,
};
use crate::errors::{AppError, Result};
use crate::types::{
    ActivityActionResponse, ActivityApplicationResponse, AgentCapability,
    UpsertActivityActionRequest, UpsertActivityApplicationRequest,
};

#[derive(Debug, Clone)]
pub struct ActivityEventInsert {
    pub observed_at: Option<chrono::DateTime<Utc>>,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub idle: bool,
    pub source: String,
    pub confidence: f32,
    pub metadata_json: String,
}

#[derive(Debug, Clone)]
pub struct ActivityUsageSpanInsert {
    pub started_at: chrono::DateTime<Utc>,
    pub ended_at: chrono::DateTime<Utc>,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub idle: bool,
    pub source: String,
    pub confidence: f32,
    pub metadata_json: String,
}

#[derive(Debug, Clone)]
pub struct ActivityClassificationIndex {
    pub actions: Vec<activity_action::Model>,
    pub applications: Vec<activity_application::Model>,
    pub aliases: Vec<activity_application_alias::Model>,
}

pub fn normalize_app_alias(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub async fn load_activity_classification_index<C: ConnectionTrait>(
    db: &C,
) -> Result<ActivityClassificationIndex> {
    let actions = activity_action::Entity::find().all(db).await?;
    let applications = activity_application::Entity::find().all(db).await?;
    let aliases = activity_application_alias::Entity::find().all(db).await?;
    Ok(ActivityClassificationIndex {
        actions,
        applications,
        aliases,
    })
}

pub async fn list_activity_actions<C: ConnectionTrait>(
    db: &C,
) -> Result<Vec<ActivityActionResponse>> {
    let actions = activity_action::Entity::find()
        .order_by_asc(activity_action::Column::SortOrder)
        .order_by_asc(activity_action::Column::ActionKey)
        .all(db)
        .await?;
    Ok(actions.into_iter().map(activity_action_response).collect())
}

pub async fn upsert_activity_action<C: ConnectionTrait>(
    db: &C,
    action_key: String,
    input: UpsertActivityActionRequest,
) -> Result<ActivityActionResponse> {
    let now = Utc::now();
    let existing = activity_action::Entity::find()
        .filter(activity_action::Column::ActionKey.eq(&action_key))
        .one(db)
        .await?;
    let model = if let Some(existing) = existing {
        let active = activity_action::ActiveModel {
            id: Set(existing.id),
            action_key: Set(existing.action_key),
            label: Set(input.label),
            status: Set(input.status),
            category: Set(input.category),
            public_label: Set(input.public_label),
            message_template: Set(input.message_template),
            enabled: Set(input.enabled),
            sort_order: Set(input.sort_order),
            created_at: Set(existing.created_at),
            updated_at: Set(now),
        };
        active.update(db).await?
    } else {
        let active = activity_action::ActiveModel {
            action_key: Set(action_key),
            label: Set(input.label),
            status: Set(input.status),
            category: Set(input.category),
            public_label: Set(input.public_label),
            message_template: Set(input.message_template),
            enabled: Set(input.enabled),
            sort_order: Set(input.sort_order),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        active.insert(db).await?
    };
    Ok(activity_action_response(model))
}

pub async fn list_activity_applications<C: ConnectionTrait>(
    db: &C,
) -> Result<Vec<ActivityApplicationResponse>> {
    let apps = activity_application::Entity::find()
        .order_by_asc(activity_application::Column::DisplayName)
        .all(db)
        .await?;
    let actions = activity_action::Entity::find().all(db).await?;
    let aliases = activity_application_alias::Entity::find()
        .order_by_asc(activity_application_alias::Column::Alias)
        .all(db)
        .await?;
    Ok(apps
        .into_iter()
        .map(|app| activity_application_response(app, &actions, &aliases))
        .collect())
}

pub async fn upsert_activity_application<C: ConnectionTrait>(
    db: &C,
    app_key: String,
    input: UpsertActivityApplicationRequest,
) -> Result<ActivityApplicationResponse> {
    let now = Utc::now();
    let default_action_id = match input.default_action_key.as_deref() {
        Some(key) if !key.trim().is_empty() => {
            let trimmed = key.trim();
            let action = activity_action::Entity::find()
                .filter(activity_action::Column::ActionKey.eq(trimmed))
                .one(db)
                .await?
                .ok_or_else(|| {
                    AppError::not_found_with_code(
                        ApiErrorCode::ActivityActionNotFound,
                        format!("activity action '{trimmed}' was not found"),
                    )
                })?;
            Some(action.id)
        }
        _ => None,
    };
    let existing = activity_application::Entity::find()
        .filter(activity_application::Column::AppKey.eq(&app_key))
        .one(db)
        .await?;
    let app = if let Some(existing) = existing {
        activity_application::ActiveModel {
            id: Set(existing.id),
            app_key: Set(existing.app_key),
            display_name: Set(input.display_name),
            default_action_id: Set(default_action_id),
            enabled: Set(input.enabled),
            created_at: Set(existing.created_at),
            updated_at: Set(now),
        }
        .update(db)
        .await?
    } else {
        activity_application::ActiveModel {
            app_key: Set(app_key),
            display_name: Set(input.display_name),
            default_action_id: Set(default_action_id),
            enabled: Set(input.enabled),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?
    };

    activity_application_alias::Entity::delete_many()
        .filter(activity_application_alias::Column::ApplicationId.eq(app.id))
        .exec(db)
        .await?;
    let alias_models = normalized_aliases(input.aliases)
        .into_iter()
        .map(
            |(alias, normalized_alias)| activity_application_alias::ActiveModel {
                application_id: Set(app.id),
                alias: Set(alias),
                normalized_alias: Set(normalized_alias),
                created_at: Set(now),
                ..Default::default()
            },
        )
        .collect::<Vec<_>>();
    if !alias_models.is_empty() {
        activity_application_alias::Entity::insert_many(alias_models)
            .exec(db)
            .await?;
    }

    let actions = activity_action::Entity::find().all(db).await?;
    let aliases = activity_application_alias::Entity::find()
        .filter(activity_application_alias::Column::ApplicationId.eq(app.id))
        .order_by_asc(activity_application_alias::Column::Alias)
        .all(db)
        .await?;
    Ok(activity_application_response(app, &actions, &aliases))
}

fn normalized_aliases(aliases: Vec<String>) -> Vec<(String, String)> {
    let mut values = Vec::new();
    for alias in aliases {
        let alias = alias.trim();
        let normalized = normalize_app_alias(alias);
        if !alias.is_empty()
            && !normalized.is_empty()
            && !values
                .iter()
                .any(|(_, existing): &(String, String)| existing == &normalized)
        {
            values.push((alias.to_string(), normalized));
        }
    }
    values
}

fn activity_action_response(action: activity_action::Model) -> ActivityActionResponse {
    ActivityActionResponse {
        id: action.id,
        action_key: action.action_key,
        label: action.label,
        status: action.status,
        category: action.category,
        public_label: action.public_label,
        message_template: action.message_template,
        enabled: action.enabled,
        sort_order: action.sort_order,
    }
}

fn activity_application_response(
    app: activity_application::Model,
    actions: &[activity_action::Model],
    aliases: &[activity_application_alias::Model],
) -> ActivityApplicationResponse {
    let default_action_key = app.default_action_id.and_then(|id| {
        actions
            .iter()
            .find(|action| action.id == id)
            .map(|action| action.action_key.clone())
    });
    ActivityApplicationResponse {
        id: app.id,
        app_key: app.app_key,
        display_name: app.display_name,
        default_action_key,
        enabled: app.enabled,
        aliases: aliases
            .iter()
            .filter(|alias| alias.application_id == app.id)
            .map(|alias| alias.alias.clone())
            .collect(),
    }
}

pub async fn create_device<C: ConnectionTrait>(
    db: &C,
    input: crate::types::CreateBeaconDeviceRequest,
) -> Result<beacon_device::Model> {
    let now = Utc::now();
    let active = beacon_device::ActiveModel {
        device_key: Set(input.device_key),
        display_name: Set(input.display_name),
        kind: Set(input.kind),
        priority: Set(input.priority),
        capabilities_json: Set("[]".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    Ok(active.insert(db).await?)
}

pub async fn list_devices<C: ConnectionTrait>(db: &C) -> Result<Vec<beacon_device::Model>> {
    Ok(beacon_device::Entity::find()
        .order_by_desc(beacon_device::Column::Priority)
        .order_by_asc(beacon_device::Column::Id)
        .all(db)
        .await?)
}

pub async fn create_device_token<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    name: String,
    token_hash: String,
) -> Result<beacon_device_token::Model> {
    let active = beacon_device_token::ActiveModel {
        device_id: Set(device_id),
        name: Set(name),
        token_hash: Set(token_hash),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    Ok(active.insert(db).await?)
}

pub async fn list_device_tokens<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
) -> Result<Vec<beacon_device_token::Model>> {
    Ok(beacon_device_token::Entity::find()
        .filter(beacon_device_token::Column::DeviceId.eq(device_id))
        .order_by_desc(beacon_device_token::Column::CreatedAt)
        .all(db)
        .await?)
}

pub async fn revoke_device_token<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    token_id: i64,
) -> Result<bool> {
    let result = beacon_device_token::Entity::update_many()
        .col_expr(
            beacon_device_token::Column::RevokedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(beacon_device_token::Column::DeviceId.eq(device_id))
        .filter(beacon_device_token::Column::Id.eq(token_id))
        .filter(beacon_device_token::Column::RevokedAt.is_null())
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}

pub async fn set_device_disabled<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    disabled: bool,
) -> Result<bool> {
    let now = Utc::now();
    let result = beacon_device::Entity::update_many()
        .col_expr(
            beacon_device::Column::DisabledAt,
            if disabled {
                sea_orm::sea_query::Expr::value(now)
            } else {
                sea_orm::sea_query::Expr::value(Option::<chrono::DateTime<Utc>>::None)
            },
        )
        .col_expr(
            beacon_device::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .filter(beacon_device::Column::Id.eq(device_id))
        .filter(if disabled {
            beacon_device::Column::DisabledAt.is_null()
        } else {
            beacon_device::Column::DisabledAt.is_not_null()
        })
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}

pub async fn find_active_device_by_token_hash<C: ConnectionTrait>(
    db: &C,
    token_hash: &str,
) -> Result<Option<(beacon_device::Model, beacon_device_token::Model)>> {
    let Some(token) = beacon_device_token::Entity::find()
        .filter(beacon_device_token::Column::TokenHash.eq(token_hash))
        .filter(beacon_device_token::Column::RevokedAt.is_null())
        .one(db)
        .await?
    else {
        return Ok(None);
    };

    let device = beacon_device::Entity::find_by_id(token.device_id)
        .filter(beacon_device::Column::DisabledAt.is_null())
        .one(db)
        .await?;

    Ok(device.map(|device| (device, token)))
}

pub async fn touch_device_token<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    token_id: i64,
) -> Result<()> {
    let now = Utc::now();
    beacon_device_token::Entity::update_many()
        .col_expr(
            beacon_device_token::Column::LastUsedAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .filter(beacon_device_token::Column::Id.eq(token_id))
        .exec(db)
        .await?;
    beacon_device::Entity::update_many()
        .col_expr(
            beacon_device::Column::LastSeenAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .filter(beacon_device::Column::Id.eq(device_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn update_device_capabilities<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    capabilities: &[AgentCapability],
) -> Result<()> {
    let now = Utc::now();
    let capability_values = capabilities
        .iter()
        .map(|capability| capability.as_str())
        .collect::<Vec<_>>();
    beacon_device::Entity::update_many()
        .col_expr(
            beacon_device::Column::CapabilitiesJson,
            sea_orm::sea_query::Expr::value(
                serde_json::to_string(&capability_values).unwrap_or_else(|_| "[]".to_string()),
            ),
        )
        .col_expr(
            beacon_device::Column::LastSeenAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .col_expr(
            beacon_device::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .filter(beacon_device::Column::Id.eq(device_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn create_activity_event<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    input: ActivityEventInsert,
) -> Result<activity_event::Model> {
    let now = Utc::now();
    let active = activity_event::ActiveModel {
        device_id: Set(device_id),
        observed_at: Set(input.observed_at.unwrap_or(now)),
        app_label: Set(input.app_label),
        project_key: Set(input.project_key),
        project_label: Set(input.project_label),
        idle: Set(input.idle),
        source: Set(input.source),
        confidence: Set(input.confidence.clamp(0.0, 1.0)),
        metadata_json: Set(input.metadata_json),
        created_at: Set(now),
        ..Default::default()
    };

    Ok(active.insert(db).await?)
}

pub async fn create_usage_spans<C: ConnectionTrait>(
    db: &C,
    device_id: i64,
    spans: Vec<ActivityUsageSpanInsert>,
) -> Result<u64> {
    if spans.is_empty() {
        return Ok(0);
    }

    let now = Utc::now();
    let active_models = spans
        .into_iter()
        .map(|span| {
            let duration_seconds = span
                .ended_at
                .signed_duration_since(span.started_at)
                .num_seconds()
                .max(1);
            activity_usage_span::ActiveModel {
                device_id: Set(device_id),
                started_at: Set(span.started_at),
                ended_at: Set(span.ended_at),
                duration_seconds: Set(duration_seconds),
                app_label: Set(span.app_label),
                project_key: Set(span.project_key),
                project_label: Set(span.project_label),
                idle: Set(span.idle),
                source: Set(span.source),
                confidence: Set(span.confidence.clamp(0.0, 1.0)),
                metadata_json: Set(span.metadata_json),
                created_at: Set(now),
                ..Default::default()
            }
        })
        .collect::<Vec<_>>();
    let inserted = active_models.len();
    activity_usage_span::Entity::insert_many(active_models)
        .exec(db)
        .await?;
    u64::try_from(inserted)
        .map_err(|_| crate::errors::AppError::Validation("usage span batch is too large".into()))
}

pub async fn latest_activity_event<C: ConnectionTrait>(
    db: &C,
) -> Result<Option<activity_event::Model>> {
    Ok(activity_event::Entity::find()
        .order_by_desc(activity_event::Column::ObservedAt)
        .one(db)
        .await?)
}

pub async fn list_activity_events_cursor<C: ConnectionTrait>(
    db: &C,
    limit: u64,
    cursor: Option<(chrono::DateTime<Utc>, i64)>,
    after: Option<chrono::DateTime<Utc>>,
) -> Result<aster_forge_api::CursorSlice<activity_event::Model>> {
    let limit = limit.clamp(1, 200);
    let mut query = activity_event::Entity::find();
    if let Some(after) = after {
        query = query.filter(activity_event::Column::ObservedAt.gte(after));
    }

    let total = query.clone().count(db).await?;
    if let Some((observed_at, id)) = cursor {
        query = query.filter(
            sea_orm::Condition::any()
                .add(activity_event::Column::ObservedAt.lt(observed_at))
                .add(
                    sea_orm::Condition::all()
                        .add(activity_event::Column::ObservedAt.eq(observed_at))
                        .add(activity_event::Column::Id.lt(id)),
                ),
        );
    }

    let items = query
        .order_by_desc(activity_event::Column::ObservedAt)
        .order_by_desc(activity_event::Column::Id)
        .limit(limit.saturating_add(1))
        .all(db)
        .await?;
    Ok(aster_forge_api::CursorSlice::from_overfetch(
        items, total, limit,
    )?)
}

pub async fn list_usage_spans_cursor<C: ConnectionTrait>(
    db: &C,
    limit: u64,
    cursor: Option<(chrono::DateTime<Utc>, i64)>,
    after: Option<chrono::DateTime<Utc>>,
) -> Result<aster_forge_api::CursorSlice<activity_usage_span::Model>> {
    let limit = limit.clamp(1, 200);
    let mut query = activity_usage_span::Entity::find();
    if let Some(after) = after {
        query = query.filter(activity_usage_span::Column::StartedAt.gte(after));
    }

    let total = query.clone().count(db).await?;
    if let Some((started_at, id)) = cursor {
        query = query.filter(
            sea_orm::Condition::any()
                .add(activity_usage_span::Column::StartedAt.lt(started_at))
                .add(
                    sea_orm::Condition::all()
                        .add(activity_usage_span::Column::StartedAt.eq(started_at))
                        .add(activity_usage_span::Column::Id.lt(id)),
                ),
        );
    }

    let items = query
        .order_by_desc(activity_usage_span::Column::StartedAt)
        .order_by_desc(activity_usage_span::Column::Id)
        .limit(limit.saturating_add(1))
        .all(db)
        .await?;
    Ok(aster_forge_api::CursorSlice::from_overfetch(
        items, total, limit,
    )?)
}

pub async fn list_usage_spans_after<C: ConnectionTrait>(
    db: &C,
    after: chrono::DateTime<Utc>,
) -> Result<Vec<activity_usage_span::Model>> {
    Ok(activity_usage_span::Entity::find()
        .filter(activity_usage_span::Column::StartedAt.gte(after))
        .order_by_desc(activity_usage_span::Column::StartedAt)
        .all(db)
        .await?)
}

pub async fn count_activity_events_after<C: ConnectionTrait>(
    db: &C,
    after: chrono::DateTime<Utc>,
) -> Result<u64> {
    Ok(activity_event::Entity::find()
        .filter(activity_event::Column::ObservedAt.gte(after))
        .count(db)
        .await?)
}

pub async fn active_manual_override<C: ConnectionTrait>(
    db: &C,
) -> Result<Option<manual_override::Model>> {
    let now = Utc::now();
    Ok(manual_override::Entity::find()
        .filter(manual_override::Column::ClearedAt.is_null())
        .filter(manual_override::Column::StartsAt.lte(now))
        .filter(
            sea_orm::Condition::any()
                .add(manual_override::Column::ExpiresAt.is_null())
                .add(manual_override::Column::ExpiresAt.gt(now)),
        )
        .order_by_desc(manual_override::Column::StartsAt)
        .order_by_desc(manual_override::Column::Id)
        .one(db)
        .await?)
}

pub async fn create_manual_override<C: ConnectionTrait>(
    db: &C,
    status: String,
    activity_label: String,
    expires_at: Option<chrono::DateTime<Utc>>,
    created_by: i64,
) -> Result<manual_override::Model> {
    clear_manual_overrides(db).await?;
    let now = Utc::now();
    Ok(manual_override::ActiveModel {
        status: Set(status),
        activity_label: Set(activity_label),
        starts_at: Set(now),
        expires_at: Set(expires_at),
        created_at: Set(now),
        created_by: Set(Some(created_by)),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn clear_manual_overrides<C: ConnectionTrait>(db: &C) -> Result<u64> {
    let result = manual_override::Entity::update_many()
        .col_expr(
            manual_override::Column::ClearedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(manual_override::Column::ClearedAt.is_null())
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
