//! Admin audit-log routes.

use actix_web::{HttpResponse, web};

use crate::api::common;
use crate::api::dto::admin::AdminAuditLogFilterQuery;
use crate::api::response::ApiResponse;

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/audit-logs",
    tag = "admin",
    params(
        aster_forge_api::LimitQuery,
        aster_forge_api::CreatedAtCursorQuery,
        AdminAuditLogFilterQuery
    ),
    responses(
        (status = 200, description = "Admin audit logs", body = inline(ApiResponse<aster_forge_api::CursorPage<crate::api::dto::admin::AdminAuditLogResponse, aster_forge_api::DateTimeIdCursor>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_audit_logs(
    state: web::Data<crate::runtime::AppState>,
    page: web::Query<aster_forge_api::LimitQuery>,
    cursor: web::Query<aster_forge_api::CreatedAtCursorQuery>,
    filter: web::Query<AdminAuditLogFilterQuery>,
) -> HttpResponse {
    match crate::services::audit_service::list_admin_audit_logs(
        state.get_ref(),
        page.into_inner(),
        cursor.into_inner(),
        filter.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}
