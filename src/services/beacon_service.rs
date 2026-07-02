//! Beacon ingest and public projection service.

use std::collections::HashMap;

use chrono::{Duration, Utc};
use rand::RngExt;

use crate::api::error_code::ApiErrorCode;
use crate::config::definitions::{
    ACTIVITY_IDLE_AFTER_SECONDS_KEY, ACTIVITY_OFFLINE_AFTER_SECONDS_KEY,
    AGENT_CONFIG_POLL_INTERVAL_SECONDS_KEY, AGENT_INCLUDE_APP_LABEL_KEY,
    AGENT_REPORT_INTERVAL_SECONDS_KEY, PRIVACY_PRIVATE_MODE_ENABLED_KEY,
    PRIVACY_PRIVATE_MODE_LABEL_KEY, VISIBILITY_PUBLIC_HISTORY_DAYS_KEY,
    VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY, VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
};
use crate::db::repository::{beacon_repo, system_config_repo};
use crate::errors::{AppError, Result};
use crate::services::activity_projection::{
    ActivityApplicationIdentity, ActivityProjectionContext, ProjectedActivity,
};
use crate::types::{
    ActivityLogEntry, ActivityMetadata, ActivitySummaryResponse, AdminActivityEventResponse,
    AdminUsageSpanResponse, AdminUsageSummaryResponse, AgentBeaconRequest,
    AgentCapabilitiesRequest, AgentCapabilitiesResponse, AgentCapability, AgentConfigResponse,
    AgentPolicyResponse, AgentUsageSpan, AgentUsageSpansRequest, ClearManualOverrideResponse,
    ManualOverrideResponse, NowResponse, ProjectSnapshot, PublicActivity, PublicActivityMessage,
    PublicMessagePart, SetManualOverrideRequest, UpdateAgentPolicyRequest,
    UpdateVisibilityPolicyRequest, UsageSpansAcceptedResponse, UsageSummaryQuery, UsageTotal,
    VisibilityPolicyResponse,
};

pub fn hash_token(token: &str) -> String {
    aster_forge_crypto::sha256_hex(token.as_bytes())
}

pub fn generate_token() -> String {
    let mut bytes = [0_u8; 32];
    rand::rng().fill(&mut bytes);
    format!("bcast_{}", aster_forge_crypto::bytes_to_hex(&bytes))
}

pub async fn create_device(
    state: &crate::runtime::AppState,
    input: crate::types::CreateBeaconDeviceRequest,
) -> Result<crate::types::CreateBeaconDeviceResponse> {
    let device = beacon_repo::create_device(state.db_handles.writer(), input).await?;
    Ok(crate::types::CreateBeaconDeviceResponse {
        id: device.id,
        device_key: device.device_key,
    })
}

pub async fn create_device_token(
    state: &crate::runtime::AppState,
    device_id: i64,
    input: crate::types::CreateBeaconDeviceTokenRequest,
) -> Result<crate::types::CreateBeaconDeviceTokenResponse> {
    let token = generate_token();
    beacon_repo::create_device_token(
        state.db_handles.writer(),
        device_id,
        input.name,
        hash_token(&token),
    )
    .await?;
    Ok(crate::types::CreateBeaconDeviceTokenResponse { token })
}

pub async fn ingest_beacon(
    state: &crate::runtime::AppState,
    token: &str,
    input: AgentBeaconRequest,
) -> Result<i64> {
    validate_agent_payload(&input)?;
    let token_hash = hash_token(token);
    let Some((device, device_token)) =
        beacon_repo::find_active_device_by_token_hash(state.db_handles.writer(), &token_hash)
            .await?
    else {
        return Err(AppError::auth_token_invalid("invalid device token"));
    };

    let insert = observed_event_insert(input);
    let event =
        beacon_repo::create_activity_event(state.db_handles.writer(), device.id, insert).await?;
    beacon_repo::touch_device_token(state.db_handles.writer(), device.id, device_token.id).await?;
    Ok(event.id)
}

pub async fn ingest_usage_spans(
    state: &crate::runtime::AppState,
    token: &str,
    input: AgentUsageSpansRequest,
) -> Result<UsageSpansAcceptedResponse> {
    validate_usage_spans_payload(&input)?;
    let token_hash = hash_token(token);
    let Some((device, device_token)) =
        beacon_repo::find_active_device_by_token_hash(state.db_handles.writer(), &token_hash)
            .await?
    else {
        return Err(AppError::auth_token_invalid("invalid device token"));
    };

    let mut spans = Vec::with_capacity(input.spans.len());
    for span in input.spans {
        spans.push(observed_span_insert(span));
    }
    let spans_accepted =
        beacon_repo::create_usage_spans(state.db_handles.writer(), device.id, spans).await?;
    beacon_repo::touch_device_token(state.db_handles.writer(), device.id, device_token.id).await?;
    Ok(UsageSpansAcceptedResponse {
        accepted: true,
        spans_accepted,
    })
}

pub async fn update_agent_capabilities(
    state: &crate::runtime::AppState,
    token: &str,
    input: AgentCapabilitiesRequest,
) -> Result<AgentCapabilitiesResponse> {
    let token_hash = hash_token(token);
    let Some((device, device_token)) =
        beacon_repo::find_active_device_by_token_hash(state.db_handles.writer(), &token_hash)
            .await?
    else {
        return Err(AppError::auth_token_invalid("invalid device token"));
    };
    let capabilities = dedupe_capabilities(input.capabilities);
    beacon_repo::update_device_capabilities(state.db_handles.writer(), device.id, &capabilities)
        .await?;
    beacon_repo::touch_device_token(state.db_handles.writer(), device.id, device_token.id).await?;
    Ok(AgentCapabilitiesResponse {
        accepted: true,
        capabilities,
    })
}

pub async fn current_public_activity(state: &crate::runtime::AppState) -> Result<NowResponse> {
    let snapshot = runtime_config_snapshot(state).await?;
    let now = Utc::now();

    if snapshot.get_bool_or(PRIVACY_PRIVATE_MODE_ENABLED_KEY, false) {
        return Ok(NowResponse {
            now: PublicActivity {
                status: "private".to_string(),
                activity_kind: "manual_note".to_string(),
                project: None,
                source: None,
                context_label: None,
                detail_badges: Vec::new(),
                message: PublicActivityMessage {
                    headline: snapshot
                        .get_string_or(PRIVACY_PRIVATE_MODE_LABEL_KEY, "Signal hidden"),
                    subline: None,
                    badges: vec!["private".to_string()],
                },
                updated_at: now,
                stale: false,
            },
        });
    }

    if let Some(override_model) =
        beacon_repo::active_manual_override(state.db_handles.reader()).await?
    {
        return Ok(NowResponse {
            now: PublicActivity {
                status: override_model.status,
                activity_kind: "manual_note".to_string(),
                project: None,
                source: None,
                context_label: None,
                detail_badges: Vec::new(),
                message: PublicActivityMessage {
                    headline: override_model.activity_label,
                    subline: None,
                    badges: Vec::new(),
                },
                updated_at: override_model.starts_at,
                stale: false,
            },
        });
    }

    let Some(event) = beacon_repo::latest_activity_event(state.db_handles.reader()).await? else {
        return Ok(NowResponse {
            now: offline(now, true),
        });
    };

    let offline_after = snapshot.get_u64_or(ACTIVITY_OFFLINE_AFTER_SECONDS_KEY, 1800) as i64;
    if now.signed_duration_since(event.observed_at) > Duration::seconds(offline_after) {
        return Ok(NowResponse {
            now: offline(event.observed_at, true),
        });
    }

    let message_parts = public_message_parts_from_snapshot(&snapshot);
    let projection_context = ActivityProjectionContext::load(state.db_handles.reader()).await?;
    Ok(NowResponse {
        now: project_public_activity(&projection_context, event, &message_parts),
    })
}

pub async fn public_activity_log(
    state: &crate::runtime::AppState,
    page: aster_forge_api::LimitQuery,
    cursor: aster_forge_api::CreatedAtCursorQuery,
) -> Result<aster_forge_api::CursorPage<ActivityLogEntry, aster_forge_api::DateTimeIdCursor>> {
    let snapshot = runtime_config_snapshot(state).await?;
    if !snapshot.get_bool_or(VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY, true)
        || snapshot.get_bool_or(PRIVACY_PRIVATE_MODE_ENABLED_KEY, false)
    {
        return Ok(aster_forge_api::CursorPage::new(
            Vec::new(),
            0,
            page.limit_or(50, 200),
            None,
        ));
    }

    let limit = page.limit_or(50, 200);
    let cursor = aster_forge_api::parse_datetime_id_cursor(
        cursor.after_created_at,
        cursor.after_id,
        "activity",
    )?;
    let after = public_history_cutoff(&snapshot);
    let message_parts = public_message_parts_from_snapshot(&snapshot);
    let projection_context = ActivityProjectionContext::load(state.db_handles.reader()).await?;
    let slice =
        beacon_repo::list_activity_events_cursor(state.db_handles.reader(), limit, cursor, after)
            .await?;
    let next_cursor = if slice.has_more {
        slice
            .items
            .last()
            .map(|item| aster_forge_api::DateTimeIdCursor {
                value: item.observed_at,
                id: item.id,
            })
    } else {
        None
    };
    let mut items = Vec::with_capacity(slice.items.len());
    for event in slice.items {
        let public_id = public_activity_event_id(event.id);
        items.push(ActivityLogEntry {
            public_id,
            activity: project_public_activity(&projection_context, event, &message_parts),
        });
    }

    Ok(aster_forge_api::CursorPage::new(
        items,
        slice.total,
        limit,
        next_cursor,
    ))
}

pub async fn public_activity_summary(
    state: &crate::runtime::AppState,
) -> Result<ActivitySummaryResponse> {
    let snapshot = runtime_config_snapshot(state).await?;
    let window_days = snapshot.get_u64_or(VISIBILITY_PUBLIC_HISTORY_DAYS_KEY, 7);
    let cutoff = Utc::now()
        .checked_sub_signed(Duration::days(i64::try_from(window_days).map_err(
            |_| AppError::Validation("public history window is too large".to_string()),
        )?))
        .ok_or_else(|| AppError::Validation("public history window overflow".to_string()))?;
    let total_events =
        beacon_repo::count_activity_events_after(state.db_handles.reader(), cutoff).await?;
    let latest = current_public_activity(state).await?.now;

    Ok(ActivitySummaryResponse {
        window_days,
        total_events: if snapshot.get_bool_or(VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY, true) {
            total_events
        } else {
            0
        },
        latest,
    })
}

pub async fn agent_config(
    state: &crate::runtime::AppState,
    token: &str,
) -> Result<AgentConfigResponse> {
    let token_hash = hash_token(token);
    if beacon_repo::find_active_device_by_token_hash(state.db_handles.reader(), &token_hash)
        .await?
        .is_none()
    {
        return Err(AppError::auth_token_invalid("invalid device token"));
    }
    let snapshot = runtime_config_snapshot(state).await?;
    Ok(AgentConfigResponse {
        poll_interval_seconds: snapshot.get_u64_or(AGENT_CONFIG_POLL_INTERVAL_SECONDS_KEY, 300),
        report_interval_seconds: snapshot.get_u64_or(AGENT_REPORT_INTERVAL_SECONDS_KEY, 30),
        offline_after_seconds: snapshot.get_u64_or(ACTIVITY_OFFLINE_AFTER_SECONDS_KEY, 1800),
        idle_after_seconds: snapshot.get_u64_or(ACTIVITY_IDLE_AFTER_SECONDS_KEY, 600),
        private_mode_enabled: snapshot.get_bool_or(PRIVACY_PRIVATE_MODE_ENABLED_KEY, false),
        include_app_label: snapshot.get_bool_or(AGENT_INCLUDE_APP_LABEL_KEY, true),
    })
}

pub async fn admin_activity_events(
    state: &crate::runtime::AppState,
    page: aster_forge_api::LimitQuery,
    cursor: aster_forge_api::CreatedAtCursorQuery,
) -> Result<
    aster_forge_api::CursorPage<AdminActivityEventResponse, aster_forge_api::DateTimeIdCursor>,
> {
    let limit = page.limit_or(50, 200);
    let cursor = aster_forge_api::parse_datetime_id_cursor(
        cursor.after_created_at,
        cursor.after_id,
        "event",
    )?;
    let slice =
        beacon_repo::list_activity_events_cursor(state.db_handles.reader(), limit, cursor, None)
            .await?;
    let projection_context = ActivityProjectionContext::load(state.db_handles.reader()).await?;
    let next_cursor = if slice.has_more {
        slice
            .items
            .last()
            .map(|item| aster_forge_api::DateTimeIdCursor {
                value: item.observed_at,
                id: item.id,
            })
    } else {
        None
    };
    let mut items = Vec::with_capacity(slice.items.len());
    for event in slice.items {
        items.push(admin_activity_event_response(&projection_context, event));
    }
    Ok(aster_forge_api::CursorPage::new(
        items,
        slice.total,
        limit,
        next_cursor,
    ))
}

pub async fn admin_usage_spans(
    state: &crate::runtime::AppState,
    page: aster_forge_api::LimitQuery,
    cursor: aster_forge_api::CreatedAtCursorQuery,
) -> Result<aster_forge_api::CursorPage<AdminUsageSpanResponse, aster_forge_api::DateTimeIdCursor>>
{
    let limit = page.limit_or(50, 200);
    let cursor = aster_forge_api::parse_datetime_id_cursor(
        cursor.after_created_at,
        cursor.after_id,
        "usage span",
    )?;
    let slice =
        beacon_repo::list_usage_spans_cursor(state.db_handles.reader(), limit, cursor, None)
            .await?;
    let projection_context = ActivityProjectionContext::load(state.db_handles.reader()).await?;
    let next_cursor = if slice.has_more {
        slice
            .items
            .last()
            .map(|item| aster_forge_api::DateTimeIdCursor {
                value: item.started_at,
                id: item.id,
            })
    } else {
        None
    };
    let mut items = Vec::with_capacity(slice.items.len());
    for span in slice.items {
        items.push(admin_usage_span_response(&projection_context, span));
    }
    Ok(aster_forge_api::CursorPage::new(
        items,
        slice.total,
        limit,
        next_cursor,
    ))
}

pub async fn admin_usage_summary(
    state: &crate::runtime::AppState,
    query: UsageSummaryQuery,
) -> Result<AdminUsageSummaryResponse> {
    if query.days == 0 || query.days > 90 {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityUsageSpanInvalid,
            "usage summary days must be between 1 and 90",
        ));
    }
    let days = i64::try_from(query.days).map_err(|_| {
        AppError::validation_with_code(
            ApiErrorCode::ActivityUsageSpanInvalid,
            "usage summary window is too large",
        )
    })?;
    let window_end = Utc::now();
    let window_start = window_end
        .checked_sub_signed(Duration::days(days))
        .ok_or_else(|| {
            AppError::validation_with_code(
                ApiErrorCode::ActivityUsageSpanInvalid,
                "usage summary window overflow",
            )
        })?;
    let spans =
        beacon_repo::list_usage_spans_after(state.db_handles.reader(), window_start).await?;
    let projection_context = ActivityProjectionContext::load(state.db_handles.reader()).await?;

    let mut total_seconds = 0_u64;
    let mut app_totals = HashMap::new();
    let mut project_totals = HashMap::new();
    let mut category_totals = HashMap::new();
    let mut status_totals = HashMap::new();

    for span in spans {
        let duration = u64::try_from(span.duration_seconds).map_err(|_| {
            AppError::validation_with_code(
                ApiErrorCode::ActivityUsageSpanInvalid,
                "usage span duration is invalid",
            )
        })?;
        total_seconds = total_seconds.saturating_add(duration);
        let projected = project_usage_span_record(&projection_context, span);
        if let Some(app) = projected.app_identity() {
            add_usage_total(&mut app_totals, app.key, app.label, duration);
        }
        if let Some(label) = projected.span.project_label.as_deref() {
            add_usage_total(
                &mut project_totals,
                projected.span.project_key.as_deref().unwrap_or(label),
                label,
                duration,
            );
        }
        if let Some(category) = projected.projection.category.as_deref() {
            add_usage_total(&mut category_totals, category, category, duration);
        }
        add_usage_total(
            &mut status_totals,
            projected.projection.status.as_str(),
            projected.projection.status.as_str(),
            duration,
        );
    }

    Ok(AdminUsageSummaryResponse {
        window_start,
        window_end,
        total_seconds,
        app_totals: sorted_usage_totals(app_totals),
        project_totals: sorted_usage_totals(project_totals),
        category_totals: sorted_usage_totals(category_totals),
        status_totals: sorted_usage_totals(status_totals),
    })
}

pub async fn visibility_policy(
    state: &crate::runtime::AppState,
) -> Result<VisibilityPolicyResponse> {
    let snapshot = runtime_config_snapshot(state).await?;
    Ok(VisibilityPolicyResponse {
        message_parts: public_message_parts_from_snapshot(&snapshot),
        public_history_enabled: snapshot.get_bool_or(VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY, true),
        public_history_days: snapshot.get_u64_or(VISIBILITY_PUBLIC_HISTORY_DAYS_KEY, 7),
        private_mode_enabled: snapshot.get_bool_or(PRIVACY_PRIVATE_MODE_ENABLED_KEY, false),
        private_mode_label: snapshot.get_string_or(PRIVACY_PRIVATE_MODE_LABEL_KEY, "Signal hidden"),
    })
}

pub async fn update_visibility_policy(
    state: &crate::runtime::AppState,
    admin_id: i64,
    input: UpdateVisibilityPolicyRequest,
) -> Result<VisibilityPolicyResponse> {
    if input.private_mode_label.trim().is_empty() {
        return Err(AppError::Validation(
            "private_mode_label cannot be empty".to_string(),
        ));
    }
    let message_parts = normalize_public_message_parts(input.message_parts);
    system_config_repo::upsert(
        state.db_handles.writer(),
        VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
        &serialize_public_message_parts(&message_parts)?,
        Some(admin_id),
    )
    .await?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        VISIBILITY_PUBLIC_HISTORY_ENABLED_KEY,
        bool_string(input.public_history_enabled),
        Some(admin_id),
    )
    .await?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        VISIBILITY_PUBLIC_HISTORY_DAYS_KEY,
        &input.public_history_days.to_string(),
        Some(admin_id),
    )
    .await?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        PRIVACY_PRIVATE_MODE_ENABLED_KEY,
        bool_string(input.private_mode_enabled),
        Some(admin_id),
    )
    .await?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        PRIVACY_PRIVATE_MODE_LABEL_KEY,
        input.private_mode_label.trim(),
        Some(admin_id),
    )
    .await?;
    visibility_policy(state).await
}

pub async fn agent_policy(state: &crate::runtime::AppState) -> Result<AgentPolicyResponse> {
    let snapshot = runtime_config_snapshot(state).await?;
    Ok(AgentPolicyResponse {
        config_poll_interval_seconds: snapshot
            .get_u64_or(AGENT_CONFIG_POLL_INTERVAL_SECONDS_KEY, 300),
        report_interval_seconds: snapshot.get_u64_or(AGENT_REPORT_INTERVAL_SECONDS_KEY, 30),
        include_app_label: snapshot.get_bool_or(AGENT_INCLUDE_APP_LABEL_KEY, true),
    })
}

pub async fn update_agent_policy(
    state: &crate::runtime::AppState,
    admin_id: i64,
    input: UpdateAgentPolicyRequest,
) -> Result<AgentPolicyResponse> {
    validate_positive_seconds(
        "config_poll_interval_seconds",
        input.config_poll_interval_seconds,
    )?;
    validate_positive_seconds("report_interval_seconds", input.report_interval_seconds)?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        AGENT_CONFIG_POLL_INTERVAL_SECONDS_KEY,
        &input.config_poll_interval_seconds.to_string(),
        Some(admin_id),
    )
    .await?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        AGENT_REPORT_INTERVAL_SECONDS_KEY,
        &input.report_interval_seconds.to_string(),
        Some(admin_id),
    )
    .await?;
    system_config_repo::upsert(
        state.db_handles.writer(),
        AGENT_INCLUDE_APP_LABEL_KEY,
        bool_string(input.include_app_label),
        Some(admin_id),
    )
    .await?;
    agent_policy(state).await
}

pub async fn active_manual_override(
    state: &crate::runtime::AppState,
) -> Result<ManualOverrideResponse> {
    Ok(manual_override_response(
        beacon_repo::active_manual_override(state.db_handles.reader()).await?,
    ))
}

pub async fn set_manual_override(
    state: &crate::runtime::AppState,
    admin_id: i64,
    input: SetManualOverrideRequest,
) -> Result<ManualOverrideResponse> {
    validate_public_label("activity", &input.activity)?;
    let expires_at = match input.expires_in_seconds {
        Some(seconds) => Some(
            Utc::now()
                .checked_add_signed(Duration::seconds(i64::try_from(seconds).map_err(|_| {
                    AppError::Validation("manual override expiry is too large".to_string())
                })?))
                .ok_or_else(|| {
                    AppError::Validation("manual override expiry overflow".to_string())
                })?,
        ),
        None => None,
    };
    let model = beacon_repo::create_manual_override(
        state.db_handles.writer(),
        input.status.as_str().to_string(),
        input.activity.trim().to_string(),
        expires_at,
        admin_id,
    )
    .await?;
    Ok(manual_override_response(Some(model)))
}

pub async fn clear_manual_override(
    state: &crate::runtime::AppState,
) -> Result<ClearManualOverrideResponse> {
    Ok(ClearManualOverrideResponse {
        cleared: beacon_repo::clear_manual_overrides(state.db_handles.writer()).await? > 0,
    })
}

fn project_public_activity(
    projection_context: &ActivityProjectionContext,
    event: crate::entities::activity_event::Model,
    message_parts: &[PublicMessagePart],
) -> PublicActivity {
    let projected = project_event_record(projection_context, event);
    let status = projected.projection.status.clone();
    let project = part_enabled(message_parts, PublicMessagePart::Project)
        .then(|| {
            projected
                .event
                .project_label
                .clone()
                .map(|label| ProjectSnapshot {
                    key: projected
                        .event
                        .project_key
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    label,
                    category: projected
                        .projection
                        .category
                        .clone()
                        .unwrap_or_else(|| "activity".to_string()),
                })
        })
        .flatten();
    let activity_kind = if part_enabled(message_parts, PublicMessagePart::Activity) {
        projected.projection.activity_kind.clone()
    } else {
        "unknown".to_string()
    };
    let source = part_enabled(message_parts, PublicMessagePart::Source)
        .then(|| safe_metadata_field(Some(projected.event.source.as_str())))
        .flatten();
    let (context_label, detail_badges) = public_message_details(
        &projected.metadata,
        projected.projection.category.as_deref(),
        projected.app_identity().map(|app| app.label),
        source.as_deref(),
        message_parts,
    );
    let message = public_activity_message(
        &status,
        &activity_kind,
        project.as_ref(),
        context_label.as_deref(),
        &detail_badges,
        projected.projection.action_public_label.as_deref(),
        projected.projection.message_template.as_deref(),
    );

    PublicActivity {
        status,
        activity_kind,
        project,
        source,
        context_label,
        detail_badges,
        message,
        updated_at: projected.event.observed_at,
        stale: false,
    }
}

fn public_activity_event_id(id: i64) -> String {
    format!("evt_{id:x}")
}

struct ProjectedActivityEvent {
    event: crate::entities::activity_event::Model,
    projection: ProjectedActivity,
    metadata: ActivityMetadata,
}

impl ProjectedActivityEvent {
    fn app_identity(&self) -> Option<ActivityApplicationIdentity<'_>> {
        self.projection
            .application_identity(self.event.app_label.as_deref())
    }
}

fn project_event_record(
    projection_context: &ActivityProjectionContext,
    event: crate::entities::activity_event::Model,
) -> ProjectedActivityEvent {
    let metadata = parse_activity_metadata(&event.metadata_json);
    let projection = projection_context.project_event(&event);
    ProjectedActivityEvent {
        event,
        projection,
        metadata,
    }
}

struct ProjectedActivityUsageSpan {
    span: crate::entities::activity_usage_span::Model,
    projection: ProjectedActivity,
    metadata: ActivityMetadata,
}

impl ProjectedActivityUsageSpan {
    fn app_identity(&self) -> Option<ActivityApplicationIdentity<'_>> {
        self.projection
            .application_identity(self.span.app_label.as_deref())
    }
}

fn project_usage_span_record(
    projection_context: &ActivityProjectionContext,
    span: crate::entities::activity_usage_span::Model,
) -> ProjectedActivityUsageSpan {
    let metadata = parse_activity_metadata(&span.metadata_json);
    let projection = projection_context.project_usage_span(&span);
    ProjectedActivityUsageSpan {
        span,
        projection,
        metadata,
    }
}

fn offline(updated_at: chrono::DateTime<Utc>, stale: bool) -> PublicActivity {
    PublicActivity {
        status: "offline".to_string(),
        activity_kind: "unknown".to_string(),
        project: None,
        source: None,
        context_label: None,
        detail_badges: Vec::new(),
        message: PublicActivityMessage {
            headline: "Offline".to_string(),
            subline: None,
            badges: vec!["offline".to_string()],
        },
        updated_at,
        stale,
    }
}

fn public_activity_message(
    status: &str,
    activity_kind: &str,
    project: Option<&ProjectSnapshot>,
    context_label: Option<&str>,
    detail_badges: &[String],
    action_label: Option<&str>,
    message_template: Option<&str>,
) -> PublicActivityMessage {
    let action = action_label
        .filter(|value| is_public_safe_message_text(value, 128))
        .unwrap_or(activity_kind);
    let project_label = project.map(|project| project.label.as_str());
    let headline = message_template
        .filter(|value| is_public_safe_message_text(value, 255))
        .map(|template| render_message_template(template, action, project_label))
        .unwrap_or_else(|| match project_label {
            Some(project) => format!("{action} - {project}"),
            None => action.to_string(),
        });
    let subline = project_label
        .map(str::to_string)
        .or_else(|| context_label.map(str::to_string));
    let mut badges = Vec::new();
    for value in std::iter::once(status)
        .chain(project_label)
        .chain(detail_badges.iter().map(String::as_str))
    {
        if is_public_safe_detail(value) && !badges.iter().any(|badge| badge == value) {
            badges.push(value.to_string());
        }
    }
    PublicActivityMessage {
        headline,
        subline,
        badges,
    }
}

fn render_message_template(template: &str, action: &str, project: Option<&str>) -> String {
    template
        .replace("{action}", action)
        .replace("{project}", project.unwrap_or("unbound task"))
}

fn public_message_details(
    metadata: &ActivityMetadata,
    category: Option<&str>,
    app_label: Option<&str>,
    source: Option<&str>,
    message_parts: &[PublicMessagePart],
) -> (Option<String>, Vec<String>) {
    let context_label = part_enabled(message_parts, PublicMessagePart::Context)
        .then(|| safe_metadata_field(metadata.safe_window_context.as_deref()))
        .flatten();
    let mut badges = Vec::new();
    if part_enabled(message_parts, PublicMessagePart::Category)
        && let Some(value) = category.filter(|value| is_public_safe_detail(value))
    {
        badges.push(value.to_string());
    }
    if part_enabled(message_parts, PublicMessagePart::App)
        && let Some(value) = app_label.filter(|value| is_public_safe_detail(value))
    {
        badges.push(value.to_string());
    }
    if part_enabled(message_parts, PublicMessagePart::Source)
        && let Some(value) = source.filter(|value| is_public_safe_detail(value))
    {
        badges.push(value.to_string());
    }
    for (part, value) in [
        (
            PublicMessagePart::BrowserContext,
            metadata.browser_context.as_deref(),
        ),
        (PublicMessagePart::GitBranch, metadata.git_branch.as_deref()),
    ] {
        if part_enabled(message_parts, part)
            && let Some(value) = safe_metadata_field(value)
            && !badges.contains(&value)
        {
            badges.push(value);
        }
    }
    (context_label, badges)
}

fn public_message_parts_from_snapshot(
    snapshot: &aster_forge_config::SyncConfigSnapshot,
) -> Vec<PublicMessagePart> {
    snapshot
        .get(VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY)
        .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .map(|parts| {
            parts
                .into_iter()
                .filter_map(|part| PublicMessagePart::parse(part.as_str()))
                .collect()
        })
        .map(normalize_public_message_parts)
        .unwrap_or_else(default_public_message_parts)
}

fn default_public_message_parts() -> Vec<PublicMessagePart> {
    vec![
        PublicMessagePart::Status,
        PublicMessagePart::Activity,
        PublicMessagePart::Project,
        PublicMessagePart::Context,
        PublicMessagePart::BrowserContext,
        PublicMessagePart::App,
        PublicMessagePart::Source,
        PublicMessagePart::GitBranch,
    ]
}

fn normalize_public_message_parts(parts: Vec<PublicMessagePart>) -> Vec<PublicMessagePart> {
    let mut normalized = Vec::new();
    for part in parts {
        if !normalized.contains(&part) {
            normalized.push(part);
        }
    }
    normalized
}

fn serialize_public_message_parts(parts: &[PublicMessagePart]) -> Result<String> {
    let values = parts.iter().map(|part| part.as_str()).collect::<Vec<_>>();
    serde_json::to_string(&values).map_err(|error| AppError::Validation(error.to_string()))
}

fn part_enabled(parts: &[PublicMessagePart], part: PublicMessagePart) -> bool {
    parts.contains(&part)
}

fn safe_metadata_field(value: Option<&str>) -> Option<String> {
    value
        .filter(|value| is_public_safe_detail(value))
        .map(str::to_string)
}

fn is_public_safe_detail(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= 80
        && trimmed.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, ' ' | '-' | '_' | '.' | '/' | ':' | '#' | '(' | ')')
        })
}

fn is_public_safe_message_text(value: &str, max_len: usize) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed.len() <= max_len
        && !trimmed
            .chars()
            .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
}

fn observed_event_insert(input: AgentBeaconRequest) -> beacon_repo::ActivityEventInsert {
    beacon_repo::ActivityEventInsert {
        observed_at: input.observed_at,
        app_label: input.app_label,
        project_key: input.project_key,
        project_label: input.project_label,
        idle: input.idle,
        source: input.source,
        confidence: input.confidence,
        metadata_json: serialize_activity_metadata(&input.metadata),
    }
}

fn observed_span_insert(input: AgentUsageSpan) -> beacon_repo::ActivityUsageSpanInsert {
    beacon_repo::ActivityUsageSpanInsert {
        started_at: input.started_at,
        ended_at: input.ended_at,
        app_label: input.app_label,
        project_key: input.project_key,
        project_label: input.project_label,
        idle: input.idle,
        source: input.source,
        confidence: input.confidence,
        metadata_json: serialize_activity_metadata(&input.metadata),
    }
}

fn serialize_activity_metadata(metadata: &ActivityMetadata) -> String {
    serde_json::to_string(metadata).unwrap_or_else(|_| "{}".to_string())
}

fn parse_activity_metadata(value: &str) -> ActivityMetadata {
    serde_json::from_str(value).unwrap_or_default()
}

fn validate_agent_payload(input: &AgentBeaconRequest) -> Result<()> {
    validate_short_label("source", &input.source, 64)?;
    validate_optional_short_label("app_label", input.app_label.as_deref(), 128)?;
    validate_optional_short_label("project_key", input.project_key.as_deref(), 128)?;
    validate_optional_short_label("project_label", input.project_label.as_deref(), 128)?;
    validate_confidence(input.confidence)?;
    validate_agent_metadata(&input.metadata)
}

fn validate_usage_spans_payload(input: &AgentUsageSpansRequest) -> Result<()> {
    if input.spans.len() > 200 {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityUsageSpanBatchTooLarge,
            "usage span batch cannot exceed 200 spans",
        ));
    }
    for span in &input.spans {
        validate_usage_span(span)?;
    }
    Ok(())
}

fn validate_usage_span(span: &AgentUsageSpan) -> Result<()> {
    validate_short_label("source", &span.source, 64)?;
    validate_optional_short_label("app_label", span.app_label.as_deref(), 128)?;
    validate_optional_short_label("project_key", span.project_key.as_deref(), 128)?;
    validate_optional_short_label("project_label", span.project_label.as_deref(), 128)?;
    validate_confidence(span.confidence)?;
    validate_agent_metadata(&span.metadata)?;

    let duration = span.ended_at.signed_duration_since(span.started_at);
    if duration.num_seconds() <= 0 {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityUsageSpanRangeInvalid,
            "usage span ended_at must be after started_at",
        ));
    }
    if duration > Duration::hours(24) {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityUsageSpanRangeInvalid,
            "usage span duration cannot exceed 24 hours",
        ));
    }
    if span.ended_at > Utc::now() + Duration::minutes(5) {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityUsageSpanRangeInvalid,
            "usage span cannot end in the future",
        ));
    }
    Ok(())
}

fn validate_agent_metadata(metadata: &ActivityMetadata) -> Result<()> {
    validate_optional_public_detail(
        "metadata.safe_window_context",
        metadata.safe_window_context.as_deref(),
    )?;
    validate_optional_public_detail(
        "metadata.browser_context",
        metadata.browser_context.as_deref(),
    )?;
    validate_optional_public_detail("metadata.git_branch", metadata.git_branch.as_deref())?;
    Ok(())
}

fn validate_optional_public_detail(name: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value
        && !is_public_safe_detail(value)
    {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityMetadataInvalid,
            format!("{name} is not safe for activity metadata"),
        ));
    }
    Ok(())
}

async fn runtime_config_snapshot(
    state: &crate::runtime::AppState,
) -> Result<aster_forge_config::SyncConfigSnapshot> {
    let configs = system_config_repo::find_all(state.db_handles.reader()).await?;
    Ok(crate::config::system_config::snapshot_from_models(configs))
}

fn public_history_cutoff(
    snapshot: &aster_forge_config::SyncConfigSnapshot,
) -> Option<chrono::DateTime<Utc>> {
    let days = snapshot.get_u64_or(VISIBILITY_PUBLIC_HISTORY_DAYS_KEY, 7);
    let days = i64::try_from(days).ok()?;
    Utc::now().checked_sub_signed(Duration::days(days))
}

fn admin_activity_event_response(
    projection_context: &ActivityProjectionContext,
    event: crate::entities::activity_event::Model,
) -> AdminActivityEventResponse {
    let projected = project_event_record(projection_context, event);
    let event = projected.event;
    let projection = projected.projection;
    AdminActivityEventResponse {
        id: event.id,
        device_id: event.device_id,
        observed_at: event.observed_at,
        status: projection.status,
        app_label: event.app_label,
        project_key: event.project_key,
        project_label: event.project_label,
        category: projection.category,
        activity_kind: projection.activity_kind,
        inference_source: projection.inference_source,
        application_key: projection.application_key,
        application_label: projection.application_label,
        action_key: projection.action_key,
        action_label: projection.action_label,
        action_public_label: projection.action_public_label,
        message_template: projection.message_template,
        source: event.source,
        confidence: event.confidence,
        metadata: projected.metadata,
        created_at: event.created_at,
    }
}

fn admin_usage_span_response(
    projection_context: &ActivityProjectionContext,
    span: crate::entities::activity_usage_span::Model,
) -> AdminUsageSpanResponse {
    let projected = project_usage_span_record(projection_context, span);
    let span = projected.span;
    let projection = projected.projection;
    AdminUsageSpanResponse {
        id: span.id,
        device_id: span.device_id,
        started_at: span.started_at,
        ended_at: span.ended_at,
        duration_seconds: u64::try_from(span.duration_seconds).unwrap_or_default(),
        status: projection.status,
        category: projection.category,
        app_label: span.app_label,
        project_key: span.project_key,
        project_label: span.project_label,
        activity_kind: projection.activity_kind,
        inference_source: projection.inference_source,
        application_key: projection.application_key,
        application_label: projection.application_label,
        action_key: projection.action_key,
        action_label: projection.action_label,
        action_public_label: projection.action_public_label,
        message_template: projection.message_template,
        source: span.source,
        confidence: span.confidence,
        metadata: projected.metadata,
        created_at: span.created_at,
    }
}

fn add_usage_total(
    totals: &mut HashMap<String, UsageTotal>,
    key: &str,
    label: &str,
    duration_seconds: u64,
) {
    let entry = totals.entry(key.to_string()).or_insert_with(|| UsageTotal {
        key: key.to_string(),
        label: label.to_string(),
        duration_seconds: 0,
    });
    entry.duration_seconds = entry.duration_seconds.saturating_add(duration_seconds);
}

fn sorted_usage_totals(totals: HashMap<String, UsageTotal>) -> Vec<UsageTotal> {
    let mut totals = totals.into_values().collect::<Vec<_>>();
    totals.sort_by(|left, right| {
        right
            .duration_seconds
            .cmp(&left.duration_seconds)
            .then_with(|| left.label.cmp(&right.label))
    });
    totals
}

fn manual_override_response(
    model: Option<crate::entities::manual_override::Model>,
) -> ManualOverrideResponse {
    let Some(model) = model else {
        return ManualOverrideResponse {
            active: false,
            id: None,
            status: None,
            activity: None,
            starts_at: None,
            expires_at: None,
        };
    };

    ManualOverrideResponse {
        active: true,
        id: Some(model.id),
        status: Some(model.status),
        activity: Some(model.activity_label),
        starts_at: Some(model.starts_at),
        expires_at: model.expires_at,
    }
}

fn validate_public_label(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(AppError::Validation(format!("{name} cannot be empty")));
    }
    if value.len() > 255 {
        return Err(AppError::Validation(format!("{name} is too long")));
    }
    Ok(())
}

fn validate_optional_short_label(name: &str, value: Option<&str>, max_len: usize) -> Result<()> {
    if let Some(value) = value {
        validate_short_label(name, value, max_len)?;
    }
    Ok(())
}

fn validate_short_label(name: &str, value: &str, max_len: usize) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{name} cannot be empty")));
    }
    if trimmed.len() > max_len {
        return Err(AppError::Validation(format!("{name} is too long")));
    }
    Ok(())
}

fn validate_confidence(value: f32) -> Result<()> {
    if !(0.0..=1.0).contains(&value) {
        return Err(AppError::validation_with_code(
            ApiErrorCode::ActivityConfidenceInvalid,
            "confidence must be between 0 and 1",
        ));
    }
    Ok(())
}

fn validate_positive_seconds(name: &str, value: u64) -> Result<()> {
    if value == 0 {
        return Err(AppError::Validation(format!(
            "{name} must be greater than 0"
        )));
    }
    Ok(())
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn dedupe_capabilities(capabilities: Vec<AgentCapability>) -> Vec<AgentCapability> {
    let mut deduped = Vec::new();
    for capability in capabilities {
        if !deduped.contains(&capability) {
            deduped.push(capability);
        }
    }
    deduped
}

#[cfg(test)]
mod tests {
    use super::{generate_token, hash_token, validate_agent_payload};
    use crate::types::{ActivityMetadata, AgentBeaconRequest};

    #[test]
    fn generated_token_is_hashable_bearer_material() {
        let token = generate_token();

        assert!(token.starts_with("bcast_"));
        assert_eq!(token.len(), 70);
        assert_eq!(hash_token(&token).len(), 64);
    }

    #[test]
    fn agent_payload_rejects_raw_private_metadata() {
        let mut request = AgentBeaconRequest {
            observed_at: None,
            app_label: Some("Code".to_string()),
            project_key: Some("beacon_cast".to_string()),
            project_label: Some("BeaconCast".to_string()),
            source: "agent".to_string(),
            idle: false,
            confidence: 1.0,
            metadata: ActivityMetadata::default(),
        };
        assert!(validate_agent_payload(&request).is_ok());

        request.metadata.git_branch = Some("secret\nbranch".to_string());
        assert!(validate_agent_payload(&request).is_err());
    }
}
