//! Admin API route aggregation.

use actix_web::{Scope, web};

use crate::api::middleware::auth::AdminSessionAuth;

pub mod activity;
pub mod audit_logs;
pub mod auth;
pub mod beacon_devices;
pub mod classification;
pub mod visibility;

pub use activity::{list_events, list_usage_spans, usage_summary};
pub use audit_logs::list_audit_logs;
pub use auth::{
    check_auth, list_sessions, login_admin, logout_admin, me, revoke_session, setup_admin,
};
pub use beacon_devices::{
    create_beacon_device, create_beacon_device_token, disable_beacon_device, enable_beacon_device,
    list_beacon_device_tokens, list_beacon_devices, revoke_beacon_device_token,
};
pub use classification::{
    list_activity_actions, list_activity_applications, upsert_activity_action,
    upsert_activity_application,
};
pub use visibility::{
    clear_manual_override, get_agent_policy, get_manual_override, get_visibility_policy,
    set_manual_override, update_agent_policy, update_visibility_policy,
};

pub fn routes() -> Scope {
    web::scope("/admin")
        .service(
            web::scope("/auth")
                .route("/check", web::get().to(check_auth))
                .route("/setup", web::post().to(setup_admin))
                .route("/login", web::post().to(login_admin))
                .service(
                    web::resource("/logout")
                        .wrap(AdminSessionAuth)
                        .route(web::post().to(logout_admin)),
                ),
        )
        .service(
            web::scope("")
                .wrap(AdminSessionAuth)
                .route("/me", web::get().to(me))
                .route("/sessions", web::get().to(list_sessions))
                .route(
                    "/sessions/{session_id}/revoke",
                    web::post().to(revoke_session),
                )
                .route("/audit-logs", web::get().to(list_audit_logs))
                .route("/events", web::get().to(list_events))
                .route("/usage-spans", web::get().to(list_usage_spans))
                .route("/usage-summary", web::get().to(usage_summary))
                .route("/activity-actions", web::get().to(list_activity_actions))
                .route(
                    "/activity-actions/{action_key}",
                    web::put().to(upsert_activity_action),
                )
                .route(
                    "/activity-applications",
                    web::get().to(list_activity_applications),
                )
                .route(
                    "/activity-applications/{app_key}",
                    web::put().to(upsert_activity_application),
                )
                .service(
                    web::resource("/visibility-policy")
                        .route(web::get().to(get_visibility_policy))
                        .route(web::put().to(update_visibility_policy)),
                )
                .service(
                    web::resource("/agent-policy")
                        .route(web::get().to(get_agent_policy))
                        .route(web::put().to(update_agent_policy)),
                )
                .service(
                    web::resource("/manual-override")
                        .route(web::get().to(get_manual_override))
                        .route(web::post().to(set_manual_override))
                        .route(web::delete().to(clear_manual_override)),
                )
                .service(
                    web::scope("/beacon-devices")
                        .route("", web::get().to(list_beacon_devices))
                        .route("", web::post().to(create_beacon_device))
                        .route("/{id}/disable", web::post().to(disable_beacon_device))
                        .route("/{id}/enable", web::post().to(enable_beacon_device))
                        .route("/{id}/tokens", web::get().to(list_beacon_device_tokens))
                        .route("/{id}/tokens", web::post().to(create_beacon_device_token))
                        .route(
                            "/{id}/tokens/{token_id}/revoke",
                            web::post().to(revoke_beacon_device_token),
                        ),
                ),
        )
}
