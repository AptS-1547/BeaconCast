//! Admin visibility-policy routes.

use actix_web::{HttpRequest, HttpResponse, web};

use crate::api::common;
use crate::api::middleware::auth::AdminSessionContext;
use crate::api::response::ApiResponse;
use crate::services::audit_service::{AdminAuditLogInput, AuditContext};
use crate::types::{
    SetManualOverrideRequest, UpdateAgentPolicyRequest, UpdateVisibilityPolicyRequest,
};

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/visibility-policy",
    tag = "admin",
    responses(
        (status = 200, description = "Current visibility policy", body = inline(ApiResponse<crate::types::VisibilityPolicyResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn get_visibility_policy(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match crate::services::beacon_service::visibility_policy(state.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    put,
    path = "/api/v1/admin/visibility-policy",
    tag = "admin",
    request_body = UpdateVisibilityPolicyRequest,
    responses(
        (status = 200, description = "Visibility policy updated", body = inline(ApiResponse<crate::types::VisibilityPolicyResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn update_visibility_policy(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    body: web::Json<UpdateVisibilityPolicyRequest>,
) -> HttpResponse {
    match crate::services::beacon_service::update_visibility_policy(
        state.get_ref(),
        session.user.id,
        body.into_inner(),
    )
    .await
    {
        Ok(response) => {
            let audit_ctx = AuditContext::from_request(&req, session.user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "visibility_policy.update",
                    entity_type: "visibility_policy",
                    entity_id: None,
                    entity_name: None,
                },
                None,
            )
            .await;
            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/agent-policy",
    tag = "admin",
    responses(
        (status = 200, description = "Current agent reporting policy", body = inline(ApiResponse<crate::types::AgentPolicyResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn get_agent_policy(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match crate::services::beacon_service::agent_policy(state.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    put,
    path = "/api/v1/admin/agent-policy",
    tag = "admin",
    request_body = UpdateAgentPolicyRequest,
    responses(
        (status = 200, description = "Agent reporting policy updated", body = inline(ApiResponse<crate::types::AgentPolicyResponse>)),
        (status = 400, description = "Invalid agent reporting policy", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn update_agent_policy(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    body: web::Json<UpdateAgentPolicyRequest>,
) -> HttpResponse {
    match crate::services::beacon_service::update_agent_policy(
        state.get_ref(),
        session.user.id,
        body.into_inner(),
    )
    .await
    {
        Ok(response) => {
            let audit_ctx = AuditContext::from_request(&req, session.user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "agent_policy.update",
                    entity_type: "agent_policy",
                    entity_id: None,
                    entity_name: None,
                },
                None,
            )
            .await;
            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/manual-override",
    tag = "admin",
    responses(
        (status = 200, description = "Current manual override", body = inline(ApiResponse<crate::types::ManualOverrideResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn get_manual_override(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match crate::services::beacon_service::active_manual_override(state.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/manual-override",
    tag = "admin",
    request_body = SetManualOverrideRequest,
    responses(
        (status = 200, description = "Manual override set", body = inline(ApiResponse<crate::types::ManualOverrideResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn set_manual_override(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    body: web::Json<SetManualOverrideRequest>,
) -> HttpResponse {
    match crate::services::beacon_service::set_manual_override(
        state.get_ref(),
        session.user.id,
        body.into_inner(),
    )
    .await
    {
        Ok(response) => {
            let audit_ctx = AuditContext::from_request(&req, session.user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "manual_override.set",
                    entity_type: "manual_override",
                    entity_id: response.id,
                    entity_name: response.activity.as_deref(),
                },
                None,
            )
            .await;
            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    delete,
    path = "/api/v1/admin/manual-override",
    tag = "admin",
    responses(
        (status = 200, description = "Manual override cleared", body = inline(ApiResponse<crate::types::ClearManualOverrideResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn clear_manual_override(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
) -> HttpResponse {
    match crate::services::beacon_service::clear_manual_override(state.get_ref()).await {
        Ok(response) => {
            if response.cleared {
                let audit_ctx = AuditContext::from_request(&req, session.user.id);
                crate::services::audit_service::record_admin_action_best_effort(
                    state.get_ref(),
                    AdminAuditLogInput {
                        ctx: &audit_ctx,
                        action: "manual_override.clear",
                        entity_type: "manual_override",
                        entity_id: None,
                        entity_name: None,
                    },
                    None,
                )
                .await;
            }
            HttpResponse::Ok().json(ApiResponse::ok(response))
        }
        Err(error) => common::app_error(error),
    }
}
