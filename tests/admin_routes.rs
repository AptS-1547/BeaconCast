//! Admin auth and beacon device route tests.

#[macro_use]
mod common;

use actix_web::{cookie::Cookie, dev::ServiceResponse, http::StatusCode, test};
use beacon_cast::api::dto::admin::{
    AdminAuditLogResponse, AdminSessionResponse, BeaconDeviceResponse, BeaconDeviceTokenResponse,
    CreateBeaconDeviceTokenAdminRequest, CreateBeaconDeviceTokenAdminResponse, RevokeResponse,
    ToggleResponse,
};
use beacon_cast::api::dto::auth::{
    AdminAuthCheckResponse, AdminAuthResponse, AdminLoginRequest, AdminLogoutResponse,
    AdminSetupRequest, AdminUserResponse,
};
use beacon_cast::api::error_code::ApiErrorCode;
use beacon_cast::api::middleware::csrf::{ADMIN_CSRF_COOKIE, ADMIN_CSRF_HEADER};
use beacon_cast::api::request_auth::ADMIN_SESSION_COOKIE;
use beacon_cast::api::response::ApiResponse;
use beacon_cast::types::{
    ActivityActionResponse, ActivityApplicationResponse, ActivityMetadata,
    AdminActivityEventResponse, AgentBeaconRequest, AgentConfigResponse, AgentPolicyResponse,
    ClearManualOverrideResponse, CreateBeaconDeviceRequest, ManualOverrideResponse, NowResponse,
    PublicMessagePart, SetManualOverrideRequest, UpdateAgentPolicyRequest,
    UpdateVisibilityPolicyRequest, UpsertActivityActionRequest, UpsertActivityApplicationRequest,
    VisibilityPolicyResponse,
};
use chrono::{DateTime, Utc};
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
    value: DateTime<Utc>,
    id: i64,
}

#[derive(Debug, Clone)]
struct AdminCookies {
    session: String,
    csrf: String,
}

trait AdminRequestExt {
    fn admin(self, cookies: &AdminCookies) -> Self;
}

impl AdminRequestExt for test::TestRequest {
    fn admin(self, cookies: &AdminCookies) -> Self {
        self.cookie(Cookie::new(
            ADMIN_SESSION_COOKIE.to_string(),
            cookies.session.clone(),
        ))
        .cookie(Cookie::new(
            ADMIN_CSRF_COOKIE.to_string(),
            cookies.csrf.clone(),
        ))
        .insert_header((ADMIN_CSRF_HEADER, cookies.csrf.clone()))
    }
}

fn extract_cookie<B>(resp: &ServiceResponse<B>, name: &str) -> String {
    resp.response()
        .cookies()
        .find(|cookie| cookie.name() == name)
        .map(|cookie| cookie.value().to_string())
        .unwrap_or_else(|| panic!("{name} cookie should be set"))
}

fn extract_admin_cookies<B>(resp: &ServiceResponse<B>) -> AdminCookies {
    AdminCookies {
        session: extract_cookie(resp, ADMIN_SESSION_COOKIE),
        csrf: extract_cookie(resp, ADMIN_CSRF_COOKIE),
    }
}

fn test_beacon_payload(confidence: f32) -> AgentBeaconRequest {
    AgentBeaconRequest {
        observed_at: None,
        app_label: Some("Code".to_string()),
        project_key: Some("beacon_cast".to_string()),
        project_label: Some("BeaconCast".to_string()),
        source: "test".to_string(),
        idle: false,
        confidence,
        metadata: ActivityMetadata::default(),
    }
}

fn test_beacon_payload_for_app(app_label: &str, confidence: f32) -> AgentBeaconRequest {
    AgentBeaconRequest {
        app_label: Some(app_label.to_string()),
        project_key: None,
        project_label: None,
        ..test_beacon_payload(confidence)
    }
}

#[actix_web::test]
async fn admin_setup_login_and_device_management_flow() {
    let state = common::setup().await;
    let app = create_test_app!(state.clone());

    let check = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/auth/check")
            .to_request(),
    )
    .await;
    assert_eq!(check.status(), StatusCode::OK);
    let check_body: ApiResponse<AdminAuthCheckResponse> = test::read_body_json(check).await;
    assert_eq!(check_body.code, ApiErrorCode::Success);
    assert!(
        !check_body
            .data
            .expect("check response should include data")
            .initialized
    );

    let setup = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/auth/setup")
            .set_json(AdminSetupRequest {
                username: "admin".to_string(),
                password: "correct-password".to_string(),
                display_name: "Admin".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(setup.status(), StatusCode::CREATED);
    let setup_cookies = extract_admin_cookies(&setup);
    let setup_body: ApiResponse<AdminAuthResponse> = test::read_body_json(setup).await;
    let setup_data = setup_body.data.expect("setup response should include data");
    assert_eq!(setup_body.code, ApiErrorCode::Success);
    assert_eq!(setup_data.user.username, "admin");
    assert!(!setup_cookies.session.is_empty());
    assert!(!setup_cookies.csrf.is_empty());

    let duplicate_setup = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/auth/setup")
            .set_json(AdminSetupRequest {
                username: "another".to_string(),
                password: "correct-password".to_string(),
                display_name: "Another".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(duplicate_setup.status(), StatusCode::BAD_REQUEST);

    let denied = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/me")
            .to_request(),
    )
    .await;
    assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    let denied_body: ApiResponse<()> = test::read_body_json(denied).await;
    assert_eq!(denied_body.code, ApiErrorCode::AuthTokenMissing);

    let login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/auth/login")
            .set_json(AdminLoginRequest {
                username: "admin".to_string(),
                password: "correct-password".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(login.status(), StatusCode::OK);
    let admin_cookies = extract_admin_cookies(&login);
    let login_body: ApiResponse<AdminAuthResponse> = test::read_body_json(login).await;
    assert_eq!(
        login_body
            .data
            .expect("login response should include data")
            .user
            .username,
        "admin"
    );

    let me = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/me")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(me.status(), StatusCode::OK);
    let me_body: ApiResponse<AdminUserResponse> = test::read_body_json(me).await;
    assert_eq!(
        me_body
            .data
            .expect("me response should include data")
            .username,
        "admin"
    );

    let create_device = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/beacon-devices")
            .admin(&admin_cookies)
            .set_json(CreateBeaconDeviceRequest {
                device_key: "macbook-main".to_string(),
                display_name: "MacBook Main".to_string(),
                kind: "desktop".to_string(),
                priority: 10,
            })
            .to_request(),
    )
    .await;
    assert_eq!(create_device.status(), StatusCode::CREATED);
    let device_body: ApiResponse<BeaconDeviceResponse> = test::read_body_json(create_device).await;
    let device = device_body
        .data
        .expect("create device response should include data");
    assert_eq!(device.device_key, "macbook-main");

    let sessions = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/sessions")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(sessions.status(), StatusCode::OK);
    let sessions_body: ApiResponse<Vec<AdminSessionResponse>> =
        test::read_body_json(sessions).await;
    let sessions_data = sessions_body
        .data
        .expect("admin sessions response should include data");
    let current_session = sessions_data
        .iter()
        .find(|session| session.current)
        .expect("current admin session should be marked");
    assert_eq!(current_session.revoked_at, None);

    let create_token = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/admin/beacon-devices/{}/tokens",
                device.id
            ))
            .admin(&admin_cookies)
            .set_json(CreateBeaconDeviceTokenAdminRequest {
                name: "local agent".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(create_token.status(), StatusCode::CREATED);
    let token_body: ApiResponse<CreateBeaconDeviceTokenAdminResponse> =
        test::read_body_json(create_token).await;
    let token = token_body
        .data
        .expect("create token response should include data")
        .token;
    assert!(token.starts_with("bcast_"));

    let list_tokens = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(&format!(
                "/api/v1/admin/beacon-devices/{}/tokens",
                device.id
            ))
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(list_tokens.status(), StatusCode::OK);
    let list_tokens_body: ApiResponse<Vec<BeaconDeviceTokenResponse>> =
        test::read_body_json(list_tokens).await;
    let tokens = list_tokens_body
        .data
        .expect("list tokens response should include data");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].name, "local agent");
    assert!(tokens[0].revoked_at.is_none());
    let token_id = tokens[0].id;

    let disable_device = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/admin/beacon-devices/{}/disable",
                device.id
            ))
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(disable_device.status(), StatusCode::OK);
    let disable_body: ApiResponse<ToggleResponse> = test::read_body_json(disable_device).await;
    assert!(
        disable_body
            .data
            .expect("disable response should include data")
            .changed
    );

    let disabled_agent_ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(test_beacon_payload(0.97))
            .to_request(),
    )
    .await;
    assert_eq!(disabled_agent_ingest.status(), StatusCode::UNAUTHORIZED);

    let enable_device = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/admin/beacon-devices/{}/enable",
                device.id
            ))
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(enable_device.status(), StatusCode::OK);
    let enable_body: ApiResponse<ToggleResponse> = test::read_body_json(enable_device).await;
    assert!(
        enable_body
            .data
            .expect("enable response should include data")
            .changed
    );

    let accepted_agent_ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(test_beacon_payload(0.97))
            .to_request(),
    )
    .await;
    assert_eq!(accepted_agent_ingest.status(), StatusCode::ACCEPTED);

    let revoke_token = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/admin/beacon-devices/{}/tokens/{token_id}/revoke",
                device.id
            ))
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(revoke_token.status(), StatusCode::OK);
    let revoke_token_body: ApiResponse<RevokeResponse> = test::read_body_json(revoke_token).await;
    assert!(
        revoke_token_body
            .data
            .expect("revoke response should include data")
            .revoked
    );

    let revoked_agent_ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {token}")))
            .set_json(test_beacon_payload(1.0))
            .to_request(),
    )
    .await;
    assert_eq!(revoked_agent_ingest.status(), StatusCode::UNAUTHORIZED);

    let events = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/events?limit=5")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(events.status(), StatusCode::OK);
    let events_body: ApiResponse<TestCursorPage<AdminActivityEventResponse, TestDateTimeIdCursor>> =
        test::read_body_json(events).await;
    let events_page = events_body
        .data
        .expect("admin events response should include data");
    assert_eq!(events_page.total, 1);
    assert_eq!(events_page.items[0].activity_kind, "writing_code");
    assert_eq!(events_page.items[0].status, "coding");
    assert_eq!(
        events_page.items[0].action_key.as_deref(),
        Some("writing_code")
    );
    assert_eq!(
        events_page.items[0].action_public_label.as_deref(),
        Some("Writing code")
    );

    let actions = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/activity-actions")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(actions.status(), StatusCode::OK);
    let actions_body: ApiResponse<Vec<ActivityActionResponse>> =
        test::read_body_json(actions).await;
    let actions_data = actions_body
        .data
        .expect("activity actions response should include data");
    assert!(actions_data.iter().any(|action| {
        action.action_key == "communicating" && action.status == "communicating"
    }));

    let upsert_action = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/activity-actions/triaging_messages")
            .admin(&admin_cookies)
            .set_json(UpsertActivityActionRequest {
                label: "Triaging messages".to_string(),
                status: "communicating".to_string(),
                category: "communication".to_string(),
                public_label: "Handling messages".to_string(),
                message_template: "{action}".to_string(),
                enabled: true,
                sort_order: 25,
            })
            .to_request(),
    )
    .await;
    assert_eq!(upsert_action.status(), StatusCode::OK);

    let update_wechat = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/activity-applications/wechat")
            .admin(&admin_cookies)
            .set_json(UpsertActivityApplicationRequest {
                display_name: "WeChat".to_string(),
                default_action_key: Some("triaging_messages".to_string()),
                enabled: true,
                aliases: vec!["WeChat".to_string(), "微信".to_string()],
            })
            .to_request(),
    )
    .await;
    assert_eq!(update_wechat.status(), StatusCode::OK);
    let update_wechat_body: ApiResponse<ActivityApplicationResponse> =
        test::read_body_json(update_wechat).await;
    let wechat_app = update_wechat_body
        .data
        .expect("activity application response should include data");
    assert_eq!(
        wechat_app.default_action_key.as_deref(),
        Some("triaging_messages")
    );

    let invalid_binding = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/activity-applications/wechat")
            .admin(&admin_cookies)
            .set_json(UpsertActivityApplicationRequest {
                display_name: "WeChat".to_string(),
                default_action_key: Some("missing_action".to_string()),
                enabled: true,
                aliases: vec!["WeChat".to_string()],
            })
            .to_request(),
    )
    .await;
    assert_eq!(invalid_binding.status(), StatusCode::NOT_FOUND);
    let invalid_binding_body: ApiResponse<()> = test::read_body_json(invalid_binding).await;
    assert_eq!(
        invalid_binding_body.code,
        ApiErrorCode::ActivityActionNotFound
    );

    let classification_token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "classification token".to_string(),
        },
    )
    .await
    .expect("create classification token")
    .token;
    let wechat_ingest = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/beacon/signals")
            .insert_header(("authorization", format!("Bearer {classification_token}")))
            .set_json(test_beacon_payload_for_app("WeChat", 0.8))
            .to_request(),
    )
    .await;
    assert_eq!(wechat_ingest.status(), StatusCode::ACCEPTED);

    let classified_events = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/events?limit=1")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(classified_events.status(), StatusCode::OK);
    let classified_events_body: ApiResponse<
        TestCursorPage<AdminActivityEventResponse, TestDateTimeIdCursor>,
    > = test::read_body_json(classified_events).await;
    let classified_event = classified_events_body
        .data
        .expect("classified events response should include data")
        .items
        .into_iter()
        .next()
        .expect("classified event should exist");
    assert_eq!(classified_event.app_label.as_deref(), Some("WeChat"));
    assert_eq!(classified_event.status, "communicating");
    assert_eq!(classified_event.activity_kind, "triaging_messages");
    assert_eq!(classified_event.application_key.as_deref(), Some("wechat"));
    assert_eq!(
        classified_event.application_label.as_deref(),
        Some("WeChat")
    );
    assert_eq!(
        classified_event.action_public_label.as_deref(),
        Some("Handling messages")
    );

    let update_triaging_action = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/activity-actions/triaging_messages")
            .admin(&admin_cookies)
            .set_json(UpsertActivityActionRequest {
                label: "Deep message triage".to_string(),
                status: "focus".to_string(),
                category: "deep_work".to_string(),
                public_label: "Sorting signal".to_string(),
                message_template: "{action}".to_string(),
                enabled: true,
                sort_order: 25,
            })
            .to_request(),
    )
    .await;
    assert_eq!(update_triaging_action.status(), StatusCode::OK);

    let reprojected_events = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/events?limit=1")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(reprojected_events.status(), StatusCode::OK);
    let reprojected_events_body: ApiResponse<
        TestCursorPage<AdminActivityEventResponse, TestDateTimeIdCursor>,
    > = test::read_body_json(reprojected_events).await;
    let reprojected_event = reprojected_events_body
        .data
        .expect("reprojected events response should include data")
        .items
        .into_iter()
        .next()
        .expect("reprojected event should exist");
    assert_eq!(reprojected_event.status, "focus");
    assert_eq!(reprojected_event.category.as_deref(), Some("deep_work"));
    assert_eq!(
        reprojected_event.action_public_label.as_deref(),
        Some("Sorting signal")
    );

    let policy_before = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/visibility-policy")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(policy_before.status(), StatusCode::OK);
    let policy_before_body: ApiResponse<VisibilityPolicyResponse> =
        test::read_body_json(policy_before).await;
    assert!(
        policy_before_body
            .data
            .expect("visibility policy response should include data")
            .message_parts
            .contains(&PublicMessagePart::Activity)
    );

    let policy_update = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/visibility-policy")
            .admin(&admin_cookies)
            .set_json(UpdateVisibilityPolicyRequest {
                message_parts: vec![PublicMessagePart::Status, PublicMessagePart::Project],
                public_history_enabled: true,
                public_history_days: 3,
                private_mode_enabled: false,
                private_mode_label: "Signal hidden".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(policy_update.status(), StatusCode::OK);
    let policy_update_body: ApiResponse<VisibilityPolicyResponse> =
        test::read_body_json(policy_update).await;
    let policy_after = policy_update_body
        .data
        .expect("visibility policy update response should include data");
    assert_eq!(
        policy_after.message_parts,
        vec![PublicMessagePart::Status, PublicMessagePart::Project]
    );
    assert_eq!(policy_after.public_history_days, 3);

    let agent_policy_before = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/agent-policy")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(agent_policy_before.status(), StatusCode::OK);
    let agent_policy_before_body: ApiResponse<AgentPolicyResponse> =
        test::read_body_json(agent_policy_before).await;
    let agent_policy_before_data = agent_policy_before_body
        .data
        .expect("agent policy response should include data");
    assert!(agent_policy_before_data.config_poll_interval_seconds > 0);
    assert!(agent_policy_before_data.report_interval_seconds > 0);
    assert!(agent_policy_before_data.include_app_label);

    let invalid_agent_policy = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/agent-policy")
            .admin(&admin_cookies)
            .set_json(UpdateAgentPolicyRequest {
                config_poll_interval_seconds: 0,
                report_interval_seconds: 10,
                include_app_label: true,
            })
            .to_request(),
    )
    .await;
    assert_eq!(invalid_agent_policy.status(), StatusCode::BAD_REQUEST);

    let agent_policy_update = test::call_service(
        &app,
        test::TestRequest::put()
            .uri("/api/v1/admin/agent-policy")
            .admin(&admin_cookies)
            .set_json(UpdateAgentPolicyRequest {
                config_poll_interval_seconds: 60,
                report_interval_seconds: 5,
                include_app_label: false,
            })
            .to_request(),
    )
    .await;
    assert_eq!(agent_policy_update.status(), StatusCode::OK);
    let agent_policy_update_body: ApiResponse<AgentPolicyResponse> =
        test::read_body_json(agent_policy_update).await;
    let agent_policy_after = agent_policy_update_body
        .data
        .expect("agent policy update response should include data");
    assert_eq!(agent_policy_after.config_poll_interval_seconds, 60);
    assert_eq!(agent_policy_after.report_interval_seconds, 5);
    assert!(!agent_policy_after.include_app_label);

    let config_token = beacon_cast::services::beacon_service::create_device_token(
        &state,
        device.id,
        beacon_cast::types::CreateBeaconDeviceTokenRequest {
            name: "agent config token".to_string(),
        },
    )
    .await
    .expect("create agent config token")
    .token;
    let agent_config_after_policy = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/agent/config")
            .insert_header(("authorization", format!("Bearer {config_token}")))
            .to_request(),
    )
    .await;
    assert_eq!(agent_config_after_policy.status(), StatusCode::OK);
    let agent_config_after_policy_body: ApiResponse<AgentConfigResponse> =
        test::read_body_json(agent_config_after_policy).await;
    let agent_config_after_policy_data = agent_config_after_policy_body
        .data
        .expect("agent config response should include data");
    assert_eq!(agent_config_after_policy_data.poll_interval_seconds, 60);
    assert_eq!(agent_config_after_policy_data.report_interval_seconds, 5);
    assert!(!agent_config_after_policy_data.include_app_label);

    let set_override = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/manual-override")
            .admin(&admin_cookies)
            .set_json(SetManualOverrideRequest {
                status: "writing".to_string(),
                activity: "manual public note".to_string(),
                expires_in_seconds: Some(600),
            })
            .to_request(),
    )
    .await;
    assert_eq!(set_override.status(), StatusCode::OK);
    let set_override_body: ApiResponse<ManualOverrideResponse> =
        test::read_body_json(set_override).await;
    assert!(
        set_override_body
            .data
            .expect("manual override response should include data")
            .active
    );

    let overridden_now = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/beacon/now")
            .to_request(),
    )
    .await;
    assert_eq!(overridden_now.status(), StatusCode::OK);
    let overridden_now_body: ApiResponse<NowResponse> = test::read_body_json(overridden_now).await;
    let overridden_now_data = overridden_now_body
        .data
        .expect("overridden now response should include data")
        .now;
    assert_eq!(overridden_now_data.status, "writing");
    assert_eq!(overridden_now_data.activity_kind, "manual_note");

    let clear_override = test::call_service(
        &app,
        test::TestRequest::delete()
            .uri("/api/v1/admin/manual-override")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(clear_override.status(), StatusCode::OK);
    let clear_override_body: ApiResponse<ClearManualOverrideResponse> =
        test::read_body_json(clear_override).await;
    assert!(
        clear_override_body
            .data
            .expect("clear override response should include data")
            .cleared
    );

    let revoke_other_session = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!(
                "/api/v1/admin/sessions/{}/revoke",
                setup_data.user.id + 10_000
            ))
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(revoke_other_session.status(), StatusCode::OK);
    let revoke_other_body: ApiResponse<RevokeResponse> =
        test::read_body_json(revoke_other_session).await;
    assert!(
        !revoke_other_body
            .data
            .expect("revoke missing session response should include data")
            .revoked
    );

    let logout = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/auth/logout")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(logout.status(), StatusCode::OK);
    let logout_body: ApiResponse<AdminLogoutResponse> = test::read_body_json(logout).await;
    assert!(
        logout_body
            .data
            .expect("logout response should include data")
            .revoked
    );

    let revoked_admin = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/me")
            .admin(&admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(revoked_admin.status(), StatusCode::UNAUTHORIZED);

    let audit_count = beacon_cast::services::audit_service::count_admin_audit_actions(
        state.db_handles.reader(),
        &[
            "admin.auth.setup",
            "admin.auth.login",
            "admin.auth.logout",
            "beacon_device.create",
            "beacon_device_token.create",
            "beacon_device_token.revoke",
            "agent_policy.update",
        ],
    )
    .await
    .expect("count audit actions");
    assert_eq!(audit_count, 7);

    let audit_login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/auth/login")
            .set_json(AdminLoginRequest {
                username: "admin".to_string(),
                password: "correct-password".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(audit_login.status(), StatusCode::OK);
    let audit_admin_cookies = extract_admin_cookies(&audit_login);
    let audit_login_body: ApiResponse<AdminAuthResponse> = test::read_body_json(audit_login).await;
    assert_eq!(
        audit_login_body
            .data
            .expect("audit login response should include data")
            .user
            .username,
        "admin"
    );

    let audit_logs = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/audit-logs?limit=3")
            .admin(&audit_admin_cookies)
            .to_request(),
    )
    .await;
    assert_eq!(audit_logs.status(), StatusCode::OK);
    let audit_logs_body: ApiResponse<TestCursorPage<AdminAuditLogResponse, TestDateTimeIdCursor>> =
        test::read_body_json(audit_logs).await;
    let audit_page = audit_logs_body
        .data
        .expect("audit log response should include data");
    assert_eq!(audit_page.limit, 3);
    assert!(audit_page.total >= 10);
    assert_eq!(audit_page.items.len(), 3);
    assert_eq!(audit_page.items[0].action, "admin.auth.login");
    let next_cursor = audit_page
        .next_cursor
        .expect("first audit page should include next cursor");
    assert!(next_cursor.id > 0);
    assert!(next_cursor.value <= Utc::now());

    let audit_denied = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/api/v1/admin/audit-logs")
            .to_request(),
    )
    .await;
    assert_eq!(audit_denied.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn admin_cookie_write_accepts_configured_trusted_origin() {
    let mut state = common::setup().await;
    let config = std::sync::Arc::make_mut(&mut state.config);
    config
        .auth
        .trusted_origins
        .push("http://127.0.0.1:5173".to_string());
    let app = create_test_app!(state.clone());

    let setup = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/auth/setup")
            .set_json(AdminSetupRequest {
                username: "admin".to_string(),
                password: "correct-password".to_string(),
                display_name: "Admin".to_string(),
            })
            .to_request(),
    )
    .await;
    assert_eq!(setup.status(), StatusCode::CREATED);
    let cookies = extract_admin_cookies(&setup);

    let create_device = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/api/v1/admin/beacon-devices")
            .insert_header(("Origin", "http://127.0.0.1:5173"))
            .admin(&cookies)
            .set_json(CreateBeaconDeviceRequest {
                device_key: "trusted-origin-device".to_string(),
                display_name: "Trusted Origin Device".to_string(),
                kind: "desktop".to_string(),
                priority: 0,
            })
            .to_request(),
    )
    .await;

    assert_eq!(create_device.status(), StatusCode::CREATED);
}
