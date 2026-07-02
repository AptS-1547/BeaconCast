//! Public and agent beacon API routes.

use actix_web::{HttpResponse, Scope, web};

use crate::api::common;
use crate::api::middleware::auth::{AgentBearerAuth, AgentBearerToken};
use crate::api::response::ApiResponse;
use crate::types::{
    AgentBeaconRequest, AgentCapabilitiesRequest, AgentUsageSpansRequest, BeaconAcceptedResponse,
};

pub fn routes() -> Scope {
    web::scope("/beacon")
        .route("/now", web::get().to(now))
        .route("/activity-log", web::get().to(activity_log))
        .route("/activity-summary", web::get().to(activity_summary))
        .service(
            web::scope("")
                .wrap(AgentBearerAuth)
                .route("/signals", web::post().to(post_beacon))
                .route("/usage-spans", web::post().to(post_usage_spans))
                .route("/agent/config", web::get().to(agent_config))
                .route("/agent/capabilities", web::put().to(put_agent_capabilities)),
        )
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/beacon/now",
    tag = "beacon",
    responses(
        (status = 200, description = "Current public activity", body = inline(ApiResponse<crate::types::NowResponse>)),
        (status = 500, description = "Service error", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn now(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match crate::services::beacon_service::current_public_activity(state.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/beacon/activity-log",
    tag = "beacon",
    params(aster_forge_api::LimitQuery),
    responses(
        (status = 200, description = "Sanitized public activity history", body = inline(ApiResponse<aster_forge_api::CursorPage<crate::types::ActivityLogEntry, aster_forge_api::DateTimeIdCursor>>)),
        (status = 500, description = "Service error", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn activity_log(
    state: web::Data<crate::runtime::AppState>,
    page: web::Query<aster_forge_api::LimitQuery>,
) -> HttpResponse {
    match crate::services::beacon_service::public_activity_log(state.get_ref(), page.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/beacon/activity-summary",
    tag = "beacon",
    responses(
        (status = 200, description = "Sanitized public activity summary", body = inline(ApiResponse<crate::types::ActivitySummaryResponse>)),
        (status = 500, description = "Service error", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn activity_summary(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match crate::services::beacon_service::public_activity_summary(state.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/beacon/signals",
    tag = "beacon",
    request_body = AgentBeaconRequest,
    responses(
        (status = 202, description = "Beacon accepted", body = inline(ApiResponse<BeaconAcceptedResponse>)),
        (status = 400, description = "Invalid beacon payload", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
        (status = 401, description = "Missing or invalid beacon device token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn post_beacon(
    state: web::Data<crate::runtime::AppState>,
    token: web::ReqData<AgentBearerToken>,
    body: web::Json<AgentBeaconRequest>,
) -> HttpResponse {
    match crate::services::beacon_service::ingest_beacon(
        state.get_ref(),
        &token.0,
        body.into_inner(),
    )
    .await
    {
        Ok(event_id) => HttpResponse::Accepted().json(ApiResponse::ok(BeaconAcceptedResponse {
            accepted: true,
            event_id,
        })),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/beacon/usage-spans",
    tag = "beacon-agent",
    request_body = AgentUsageSpansRequest,
    responses(
        (status = 202, description = "Usage spans accepted", body = inline(ApiResponse<crate::types::UsageSpansAcceptedResponse>)),
        (status = 400, description = "Invalid usage span payload", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
        (status = 401, description = "Missing or invalid beacon device token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn post_usage_spans(
    state: web::Data<crate::runtime::AppState>,
    token: web::ReqData<AgentBearerToken>,
    body: web::Json<AgentUsageSpansRequest>,
) -> HttpResponse {
    match crate::services::beacon_service::ingest_usage_spans(
        state.get_ref(),
        &token.0,
        body.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Accepted().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/beacon/agent/config",
    tag = "beacon-agent",
    responses(
        (status = 200, description = "Agent runtime policy", body = inline(ApiResponse<crate::types::AgentConfigResponse>)),
        (status = 401, description = "Missing or invalid beacon device token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn agent_config(
    state: web::Data<crate::runtime::AppState>,
    token: web::ReqData<AgentBearerToken>,
) -> HttpResponse {
    match crate::services::beacon_service::agent_config(state.get_ref(), &token.0).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    put,
    path = "/api/v1/beacon/agent/capabilities",
    tag = "beacon-agent",
    request_body = AgentCapabilitiesRequest,
    responses(
        (status = 200, description = "Agent capabilities accepted", body = inline(ApiResponse<crate::types::AgentCapabilitiesResponse>)),
        (status = 401, description = "Missing or invalid beacon device token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>))
    )
)]
pub async fn put_agent_capabilities(
    state: web::Data<crate::runtime::AppState>,
    token: web::ReqData<AgentBearerToken>,
    body: web::Json<AgentCapabilitiesRequest>,
) -> HttpResponse {
    match crate::services::beacon_service::update_agent_capabilities(
        state.get_ref(),
        &token.0,
        body.into_inner(),
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}
