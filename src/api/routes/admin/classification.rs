//! Admin activity classification configuration routes.

use actix_web::{HttpRequest, HttpResponse, web};

use crate::api::common;
use crate::api::middleware::auth::AdminSessionContext;
use crate::api::response::ApiResponse;
use crate::db::repository::beacon_repo;
use crate::services::audit_service::{AdminAuditLogInput, AuditContext};
use crate::types::{UpsertActivityActionRequest, UpsertActivityApplicationRequest};

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/activity-actions",
    tag = "admin",
    responses(
        (status = 200, description = "Activity actions", body = inline(ApiResponse<Vec<crate::types::ActivityActionResponse>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_activity_actions(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match beacon_repo::list_activity_actions(state.db_handles.reader()).await {
        Ok(actions) => HttpResponse::Ok().json(ApiResponse::ok(actions)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    put,
    path = "/api/v1/admin/activity-actions/{action_key}",
    tag = "admin",
    request_body = UpsertActivityActionRequest,
    params(("action_key" = String, Path, description = "Activity action key")),
    responses(
        (status = 200, description = "Activity action saved", body = inline(ApiResponse<crate::types::ActivityActionResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn upsert_activity_action(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    action_key: web::Path<String>,
    body: web::Json<UpsertActivityActionRequest>,
) -> HttpResponse {
    let action_key = action_key.into_inner();
    match beacon_repo::upsert_activity_action(
        state.db_handles.writer(),
        action_key.clone(),
        body.into_inner(),
    )
    .await
    {
        Ok(action) => {
            record_classification_audit(
                &req,
                state.get_ref(),
                session.user.id,
                "activity_action.upsert",
                action.id,
                &action.action_key,
            )
            .await;
            HttpResponse::Ok().json(ApiResponse::ok(action))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/activity-applications",
    tag = "admin",
    responses(
        (status = 200, description = "Activity applications", body = inline(ApiResponse<Vec<crate::types::ActivityApplicationResponse>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_activity_applications(
    state: web::Data<crate::runtime::AppState>,
) -> HttpResponse {
    match beacon_repo::list_activity_applications(state.db_handles.reader()).await {
        Ok(applications) => HttpResponse::Ok().json(ApiResponse::ok(applications)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    put,
    path = "/api/v1/admin/activity-applications/{app_key}",
    tag = "admin",
    request_body = UpsertActivityApplicationRequest,
    params(("app_key" = String, Path, description = "Activity application key")),
    responses(
        (status = 200, description = "Activity application saved", body = inline(ApiResponse<crate::types::ActivityApplicationResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn upsert_activity_application(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    app_key: web::Path<String>,
    body: web::Json<UpsertActivityApplicationRequest>,
) -> HttpResponse {
    let app_key = app_key.into_inner();
    match beacon_repo::upsert_activity_application(
        state.db_handles.writer(),
        app_key.clone(),
        body.into_inner(),
    )
    .await
    {
        Ok(application) => {
            record_classification_audit(
                &req,
                state.get_ref(),
                session.user.id,
                "activity_application.upsert",
                application.id,
                &application.app_key,
            )
            .await;
            HttpResponse::Ok().json(ApiResponse::ok(application))
        }
        Err(error) => common::app_error(error),
    }
}

async fn record_classification_audit(
    req: &HttpRequest,
    state: &crate::runtime::AppState,
    admin_id: i64,
    action: &'static str,
    entity_id: i64,
    entity_name: &str,
) {
    let audit_ctx = AuditContext::from_request(req, admin_id);
    crate::services::audit_service::record_admin_action_best_effort(
        state,
        AdminAuditLogInput {
            ctx: &audit_ctx,
            action,
            entity_type: "activity_classification",
            entity_id: Some(entity_id),
            entity_name: Some(entity_name),
        },
        None,
    )
    .await;
}
