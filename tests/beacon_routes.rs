//! Beacon public and agent route tests.

#[macro_use]
mod common;

use actix_web::{http::StatusCode, test};
use beacon_cast::api::error_code::ApiErrorCode;
use beacon_cast::api::response::ApiResponse;
use beacon_cast::types::{
    ActivityLogEntry, ActivityMetadata, ActivitySummaryResponse, AgentBeaconRequest,
    AgentCapabilitiesRequest, AgentCapabilitiesResponse, AgentCapability, AgentConfigResponse,
    AgentUsageSpan, AgentUsageSpansRequest, BeaconAcceptedResponse, NowResponse, PublicMessagePart,
    UpsertActivityActionRequest, UsageSpansAcceptedResponse, UsageSummaryQuery,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct TestCursorPage<T, C> {
    items: Vec<T>,
    total: u64,
    limit: u64,
    next_cursor: Option<C>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TestDateTimeIdCursor {
    value: chrono::DateTime<chrono::Utc>,
    id: i64,
}

#[actix_web::test]
async fn now_returns_offline_without_recent_signal() {
    let state = common::setup().await;
    let app = create_test_app!(state.clone());

    let response = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: ApiResponse<NowResponse> = test::read_body_json(response).await;
    let now = body.data.expect("now response should include data").now;
    assert_eq!(body.code, ApiErrorCode::Success);
    assert_eq!(now.status, "offline");
    assert!(now.stale);
}

#[actix_web::test]
async fn beacon_ingest_requires_bearer_token() {
    let state = common::setup().await;
    let app = create_test_app!(state.clone());

    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .set_json(valid_beacon_payload())
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body: ApiResponse<()> = test::read_body_json(response).await;
    assert_eq!(body.code, ApiErrorCode::AuthTokenMissing);
}

#[actix_web::test]
async fn beacon_ingest_updates_public_now_projection() {
    let state = common::setup().await;
    let device = beacon_cast::services::beacon_service::create_device(
        &state,
        beacon_cast::types::CreateBeaconDeviceRequest {
            device_key: "macbook-main".to_string(),
            display_name: "MacBook Main".to_string(),
            kind: "desktop".to_string(),
            priority: 10,
        },
    )
    .await
    .expect("create beacon device");
    let token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "test token".to_string(),
        },
    )
    .await
    .expect("create beacon device token")
    .token;
    let app = create_test_app!(state.clone());

    let capabilities = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/beacon/agent/capabilities")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(AgentCapabilitiesRequest {
                capabilities: vec![AgentCapability::FrontmostApp, AgentCapability::FrontmostApp],
            })
            .to_request(),
    )
    .await;
    assert_eq!(capabilities.status(), StatusCode::OK);
    let capabilities_body: ApiResponse<AgentCapabilitiesResponse> =
        test::read_body_json(capabilities).await;
    let capabilities_data = capabilities_body
        .data
        .expect("capabilities response should include data");
    assert!(capabilities_data.accepted);
    assert_eq!(
        capabilities_data.capabilities,
        vec![AgentCapability::FrontmostApp]
    );

    let devices = beacon_cast::db::repository::beacon_repo::list_devices(state.db_handles.reader())
        .await
        .expect("list devices after capabilities");
    assert_eq!(devices[0].capabilities_json, r#"["frontmost_app"]"#);

    let ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(valid_beacon_payload())
            .to_request(),
    )
    .await;
    assert_eq!(ingest.status(), StatusCode::ACCEPTED);
    let ingest_body: ApiResponse<BeaconAcceptedResponse> = test::read_body_json(ingest).await;
    assert_eq!(ingest_body.code, ApiErrorCode::Success);
    assert!(
        ingest_body
            .data
            .expect("beacon response should include data")
            .accepted
    );

    let now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(now.status(), StatusCode::OK);
    let now_body: ApiResponse<NowResponse> = test::read_body_json(now).await;
    let now_data = now_body.data.expect("now response should include data").now;
    assert_eq!(now_body.code, ApiErrorCode::Success);
    assert_eq!(now_data.status, "coding");
    assert_eq!(now_data.activity_kind, "writing_code");
    assert_eq!(
        now_data.context_label.as_deref(),
        Some("BeaconCast workspace")
    );
    assert!(now_data.detail_badges.contains(&"Code".to_string()));
    assert_eq!(
        now_data.project.expect("project should be public").key,
        "beacon_cast"
    );

    let log = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/activity-log?limit=10")
            .to_request(),
    )
    .await;
    assert_eq!(log.status(), StatusCode::OK);
    let log_body: ApiResponse<TestCursorPage<ActivityLogEntry, TestDateTimeIdCursor>> =
        test::read_body_json(log).await;
    let log_page = log_body
        .data
        .expect("activity log response should include data");
    assert_eq!(log_page.total, 1);
    assert_eq!(log_page.items[0].activity.activity_kind, "writing_code");
    assert!(log_page.next_cursor.is_none());

    beacon_cast::db::repository::system_config_repo::upsert(
        state.db_handles.writer(),
        beacon_cast::config::definitions::VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
        &message_parts_json(&[
            PublicMessagePart::Status,
            PublicMessagePart::Activity,
            PublicMessagePart::Project,
            PublicMessagePart::Context,
            PublicMessagePart::BrowserContext,
            PublicMessagePart::App,
            PublicMessagePart::GitBranch,
        ]),
        None,
    )
    .await
    .expect("set rich public message parts");
    let rich_now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(rich_now.status(), StatusCode::OK);
    let rich_now_body: ApiResponse<NowResponse> = test::read_body_json(rich_now).await;
    let rich_now_data = rich_now_body
        .data
        .expect("rich now response should include data")
        .now;
    assert_eq!(
        rich_now_data.context_label.as_deref(),
        Some("BeaconCast workspace")
    );
    assert!(rich_now_data.detail_badges.contains(&"Code".to_string()));
    assert!(
        !rich_now_data
            .detail_badges
            .contains(&"auto_git_project".to_string())
    );
    assert_eq!(rich_now_data.activity_kind, "writing_code");

    let summary = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/activity-summary")
            .to_request(),
    )
    .await;
    assert_eq!(summary.status(), StatusCode::OK);
    let summary_body: ApiResponse<ActivitySummaryResponse> = test::read_body_json(summary).await;
    let summary_data = summary_body
        .data
        .expect("activity summary response should include data");
    assert_eq!(summary_data.total_events, 1);
    assert_eq!(summary_data.latest.activity_kind, "writing_code");

    beacon_cast::db::repository::system_config_repo::upsert(
        state.db_handles.writer(),
        beacon_cast::config::definitions::VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
        &message_parts_json(&[PublicMessagePart::Status]),
        None,
    )
    .await
    .expect("set status-only public message parts");
    let status_only_now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(status_only_now.status(), StatusCode::OK);
    let status_only_body: ApiResponse<NowResponse> = test::read_body_json(status_only_now).await;
    let status_only_data = status_only_body
        .data
        .expect("status-only now response should include data")
        .now;
    assert_eq!(status_only_data.activity_kind, "unknown");
    assert!(status_only_data.project.is_none());

    beacon_cast::db::repository::system_config_repo::upsert(
        state.db_handles.writer(),
        beacon_cast::config::definitions::VISIBILITY_PUBLIC_MESSAGE_PARTS_KEY,
        "[]",
        None,
    )
    .await
    .expect("set hidden public message parts");
    let hidden_now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(hidden_now.status(), StatusCode::OK);
    let hidden_body: ApiResponse<NowResponse> = test::read_body_json(hidden_now).await;
    let hidden_data = hidden_body
        .data
        .expect("hidden now response should include data")
        .now;
    assert_eq!(hidden_data.activity_kind, "unknown");
    assert!(hidden_data.project.is_none());
    assert!(hidden_data.detail_badges.is_empty());

    let agent_config = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/agent/config")
            .insert_header(("authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_eq!(agent_config.status(), StatusCode::OK);
    let agent_config_body: ApiResponse<AgentConfigResponse> =
        test::read_body_json(agent_config).await;
    let agent_config_data = agent_config_body
        .data
        .expect("agent config response should include data");
    assert!(agent_config_data.poll_interval_seconds > 0);
    assert!(agent_config_data.report_interval_seconds > 0);
    assert!(agent_config_data.include_app_label);
}

#[actix_web::test]
async fn public_activity_log_respects_configured_public_history_limit() {
    let state = common::setup().await;
    let device = beacon_cast::services::beacon_service::create_device(
        &state,
        beacon_cast::types::CreateBeaconDeviceRequest {
            device_key: "public-limit-device".to_string(),
            display_name: "Public Limit Device".to_string(),
            kind: "desktop".to_string(),
            priority: 10,
        },
    )
    .await
    .expect("create beacon device");
    let token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "public limit token".to_string(),
        },
    )
    .await
    .expect("create beacon device token")
    .token;
    beacon_cast::db::repository::system_config_repo::upsert(
        state.db_handles.writer(),
        beacon_cast::config::definitions::VISIBILITY_PUBLIC_HISTORY_LIMIT_KEY,
        "2",
        None,
    )
    .await
    .expect("set public history limit");
    let app = create_test_app!(state.clone());

    for offset in [3, 2, 1] {
        let mut signal = valid_beacon_payload();
        signal.observed_at = Some(chrono::Utc::now() - chrono::Duration::seconds(offset));
        let ingest = test::call_service(
            &app,
            test::TestRequest::post()
                .uri("/api/v1/beacon/signals")
                .insert_header(("authorization", format!("Bearer {token}")))
                .set_json(signal)
                .to_request(),
        )
        .await;
        assert_eq!(ingest.status(), StatusCode::ACCEPTED);
    }

    let log = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/activity-log?limit=10")
            .to_request(),
    )
    .await;
    assert_eq!(log.status(), StatusCode::OK);
    let log_body: ApiResponse<TestCursorPage<ActivityLogEntry, TestDateTimeIdCursor>> =
        test::read_body_json(log).await;
    let log_page = log_body
        .data
        .expect("activity log response should include data");
    assert_eq!(log_page.limit, 2);
    assert_eq!(log_page.total, 2);
    assert_eq!(log_page.items.len(), 2);
    assert!(log_page.next_cursor.is_none());
}

#[actix_web::test]
async fn unconfigured_app_does_not_inherit_hardcoded_activity() {
    let state = common::setup().await;
    let device = beacon_cast::services::beacon_service::create_device(
        &state,
        beacon_cast::types::CreateBeaconDeviceRequest {
            device_key: "unknown-app-device".to_string(),
            display_name: "Unknown App Device".to_string(),
            kind: "desktop".to_string(),
            priority: 10,
        },
    )
    .await
    .expect("create beacon device");
    let token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "unknown app token".to_string(),
        },
    )
    .await
    .expect("create beacon device token")
    .token;
    let app = create_test_app!(state.clone());
    let mut signal = valid_beacon_payload();
    signal.app_label = Some("Totally Unknown App".to_string());
    signal.project_key = None;
    signal.project_label = None;

    let ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(signal)
            .to_request(),
    )
    .await;
    assert_eq!(ingest.status(), StatusCode::ACCEPTED);

    let now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(now.status(), StatusCode::OK);
    let now_body: ApiResponse<NowResponse> = test::read_body_json(now).await;
    let now_data = now_body.data.expect("now response should include data").now;
    assert_eq!(now_data.status, "unclassified");
    assert_eq!(now_data.activity_kind, "unclassified_activity");
    assert_eq!(now_data.message.headline, "unclassified_activity");
    assert!(now_data.project.is_none());
}

#[actix_web::test]
async fn public_message_allows_admin_configured_unicode_labels() {
    let state = common::setup().await;
    let device = beacon_cast::services::beacon_service::create_device(
        &state,
        beacon_cast::types::CreateBeaconDeviceRequest {
            device_key: "unicode-label-device".to_string(),
            display_name: "Unicode Label Device".to_string(),
            kind: "desktop".to_string(),
            priority: 10,
        },
    )
    .await
    .expect("create beacon device");
    let token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "unicode label token".to_string(),
        },
    )
    .await
    .expect("create beacon device token")
    .token;
    beacon_cast::db::repository::beacon_repo::upsert_activity_action(
        state.db_handles.writer(),
        "writing_code".to_string(),
        UpsertActivityActionRequest {
            label: "Writing code".to_string(),
            status: "coding".to_string(),
            category: "coding".to_string(),
            public_label: "写代码".to_string(),
            message_template: "{action}".to_string(),
            enabled: true,
            sort_order: 10,
        },
    )
    .await
    .expect("update writing_code public label");
    let app = create_test_app!(state.clone());

    let ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(valid_beacon_payload())
            .to_request(),
    )
    .await;
    assert_eq!(ingest.status(), StatusCode::ACCEPTED);

    let now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(now.status(), StatusCode::OK);
    let now_body: ApiResponse<NowResponse> = test::read_body_json(now).await;
    let now_data = now_body.data.expect("now response should include data").now;
    assert_eq!(now_data.message.headline, "写代码");
    assert_eq!(now_data.message.badges[0], "coding");
}

#[actix_web::test]
async fn existing_events_use_current_classification_config_on_read() {
    let state = common::setup().await;
    let device = beacon_cast::services::beacon_service::create_device(
        &state,
        beacon_cast::types::CreateBeaconDeviceRequest {
            device_key: "read-time-classification-device".to_string(),
            display_name: "Read Time Classification Device".to_string(),
            kind: "desktop".to_string(),
            priority: 10,
        },
    )
    .await
    .expect("create beacon device");
    let token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "read time classification token".to_string(),
        },
    )
    .await
    .expect("create beacon device token")
    .token;
    let app = create_test_app!(state.clone());

    let ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(valid_beacon_payload())
            .to_request(),
    )
    .await;
    assert_eq!(ingest.status(), StatusCode::ACCEPTED);

    beacon_cast::db::repository::beacon_repo::upsert_activity_action(
        state.db_handles.writer(),
        "writing_code".to_string(),
        UpsertActivityActionRequest {
            label: "Writing code".to_string(),
            status: "coding".to_string(),
            category: "coding".to_string(),
            public_label: "写代码".to_string(),
            message_template: "{action}".to_string(),
            enabled: true,
            sort_order: 10,
        },
    )
    .await
    .expect("update writing_code public label after event insert");

    let now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(now.status(), StatusCode::OK);
    let now_body: ApiResponse<NowResponse> = test::read_body_json(now).await;
    let now_data = now_body.data.expect("now response should include data").now;
    assert_eq!(now_data.message.headline, "写代码");

    let log = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/activity-log?limit=10")
            .to_request(),
    )
    .await;
    assert_eq!(log.status(), StatusCode::OK);
    let log_body: ApiResponse<TestCursorPage<ActivityLogEntry, TestDateTimeIdCursor>> =
        test::read_body_json(log).await;
    let log_page = log_body
        .data
        .expect("activity log response should include data");
    assert_eq!(log_page.items[0].activity.message.headline, "写代码");
}

fn message_parts_json(parts: &[PublicMessagePart]) -> String {
    let values = parts.iter().map(|part| part.as_str()).collect::<Vec<_>>();
    serde_json::to_string(&values).expect("message parts should serialize")
}

#[actix_web::test]
async fn usage_spans_are_ingested_and_summarized_without_raw_metadata() {
    let state = common::setup().await;
    let device = beacon_cast::services::beacon_service::create_device(
        &state,
        beacon_cast::types::CreateBeaconDeviceRequest {
            device_key: "macbook-usage".to_string(),
            display_name: "MacBook Usage".to_string(),
            kind: "desktop".to_string(),
            priority: 10,
        },
    )
    .await
    .expect("create beacon device");
    let token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "usage token".to_string(),
        },
    )
    .await
    .expect("create beacon device token")
    .token;
    let app = create_test_app!(state.clone());
    let ended_at = chrono::Utc::now();
    let started_at = ended_at - chrono::Duration::seconds(90);

    let accepted = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/usage-spans")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(AgentUsageSpansRequest {
                spans: vec![usage_span(started_at, ended_at)],
            })
            .to_request(),
    )
    .await;
    assert_eq!(accepted.status(), StatusCode::ACCEPTED);
    let accepted_body: ApiResponse<UsageSpansAcceptedResponse> =
        test::read_body_json(accepted).await;
    assert_eq!(
        accepted_body
            .data
            .expect("usage spans response should include data")
            .spans_accepted,
        1
    );

    let summary = beacon_cast::services::beacon_service::admin_usage_summary(
        &state,
        UsageSummaryQuery { days: 1 },
    )
    .await
    .expect("usage summary should load");
    assert_eq!(summary.total_seconds, 90);
    assert_eq!(summary.app_totals[0].key, "code");
    assert_eq!(summary.app_totals[0].label, "Code");
    assert_eq!(summary.app_totals[0].duration_seconds, 90);
    assert_eq!(summary.project_totals[0].key, "beacon_cast");

    let mut unsafe_span = usage_span(started_at, ended_at);
    unsafe_span.metadata.git_branch = Some("secret\nbranch".to_string());
    let rejected = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/usage-spans")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(AgentUsageSpansRequest {
                spans: vec![unsafe_span],
            })
            .to_request(),
    )
    .await;
    assert_eq!(rejected.status(), StatusCode::BAD_REQUEST);
    let rejected_body: ApiResponse<()> = test::read_body_json(rejected).await;
    assert_eq!(rejected_body.code, ApiErrorCode::ActivityMetadataInvalid);
}

fn valid_beacon_payload() -> AgentBeaconRequest {
    AgentBeaconRequest {
        observed_at: None,
        app_label: Some("Code".to_string()),
        project_key: Some("beacon_cast".to_string()),
        project_label: Some("BeaconCast".to_string()),
        source: "test".to_string(),
        idle: false,
        confidence: 0.92,
        metadata: ActivityMetadata {
            safe_window_context: Some("BeaconCast workspace".to_string()),
            browser_context: None,
            git_branch: Some("main".to_string()),
        },
    }
}

fn usage_span(
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: chrono::DateTime<chrono::Utc>,
) -> AgentUsageSpan {
    AgentUsageSpan {
        started_at,
        ended_at,
        app_label: Some("Code".to_string()),
        project_key: Some("beacon_cast".to_string()),
        project_label: Some("BeaconCast".to_string()),
        source: "frontmost_app".to_string(),
        idle: false,
        confidence: 0.9,
        metadata: ActivityMetadata::default(),
    }
}
