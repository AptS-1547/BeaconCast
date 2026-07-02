//! Admin activity-event routes.

use actix_web::{HttpResponse, web};

use crate::api::common;
use crate::api::response::ApiResponse;

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/events",
    tag = "admin",
    params(aster_forge_api::LimitQuery, aster_forge_api::CreatedAtCursorQuery),
    responses(
        (status = 200, description = "Raw admin-visible activity events", body = inline(ApiResponse<aster_forge_api::CursorPage<crate::types::AdminActivityEventResponse, aster_forge_api::DateTimeIdCursor>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_events(
    state: web::Data<crate::runtime::AppState>,
    page: web::Query<aster_forge_api::LimitQuery>,
    cursor: web::Query<aster_forge_api::CreatedAtCursorQuery>,
) -> HttpResponse {
    match crate::services::beacon_service::admin_activity_events(
        state.get_ref(),
        page.into_inner(),
        cursor.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/usage-spans",
    tag = "admin",
    params(aster_forge_api::LimitQuery, aster_forge_api::CreatedAtCursorQuery),
    responses(
        (status = 200, description = "Admin-visible usage spans", body = inline(ApiResponse<aster_forge_api::CursorPage<crate::types::AdminUsageSpanResponse, aster_forge_api::DateTimeIdCursor>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_usage_spans(
    state: web::Data<crate::runtime::AppState>,
    page: web::Query<aster_forge_api::LimitQuery>,
    cursor: web::Query<aster_forge_api::CreatedAtCursorQuery>,
) -> HttpResponse {
    match crate::services::beacon_service::admin_usage_spans(
        state.get_ref(),
        page.into_inner(),
        cursor.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/usage-summary",
    tag = "admin",
    params(crate::types::UsageSummaryQuery),
    responses(
        (status = 200, description = "Usage totals for the selected window", body = inline(ApiResponse<crate::types::AdminUsageSummaryResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn usage_summary(
    state: web::Data<crate::runtime::AppState>,
    query: web::Query<crate::types::UsageSummaryQuery>,
) -> HttpResponse {
    match crate::services::beacon_service::admin_usage_summary(state.get_ref(), query.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}
