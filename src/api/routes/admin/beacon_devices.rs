//! Admin beacon-device routes.

use actix_web::{HttpRequest, HttpResponse, web};

use crate::api::common;
use crate::api::dto::admin::{
    BeaconDeviceResponse, BeaconDeviceTokenResponse, CreateBeaconDeviceTokenAdminRequest,
    CreateBeaconDeviceTokenAdminResponse, RevokeResponse, ToggleResponse,
};
use crate::api::middleware::auth::AdminSessionContext;
use crate::api::response::ApiResponse;
use crate::db::repository::beacon_repo;
use crate::services::audit_service::{AdminAuditLogInput, AuditContext};
use crate::types::CreateBeaconDeviceRequest;

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/beacon-devices",
    tag = "admin",
    responses(
        (status = 200, description = "Beacon devices", body = inline(ApiResponse<Vec<BeaconDeviceResponse>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_beacon_devices(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match beacon_repo::list_devices(state.db_handles.reader()).await {
        Ok(devices) => HttpResponse::Ok().json(ApiResponse::ok(
            devices
                .into_iter()
                .map(BeaconDeviceResponse::from)
                .collect::<Vec<_>>(),
        )),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/beacon-devices",
    tag = "admin",
    request_body = CreateBeaconDeviceRequest,
    responses(
        (status = 201, description = "Beacon device created", body = inline(ApiResponse<BeaconDeviceResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn create_beacon_device(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    body: web::Json<CreateBeaconDeviceRequest>,
) -> HttpResponse {
    match beacon_repo::create_device(state.db_handles.writer(), body.into_inner()).await {
        Ok(device) => {
            let audit_ctx = AuditContext::from_request(&req, session.user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "beacon_device.create",
                    entity_type: "beacon_device",
                    entity_id: Some(device.id),
                    entity_name: Some(&device.device_key),
                },
                None,
            )
            .await;
            HttpResponse::Created().json(ApiResponse::ok(BeaconDeviceResponse::from(device)))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/beacon-devices/{id}/disable",
    tag = "admin",
    params(("id" = i64, Path, description = "Beacon device id")),
    responses(
        (status = 200, description = "Beacon device disabled", body = inline(ApiResponse<ToggleResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn disable_beacon_device(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    id: web::Path<i64>,
) -> HttpResponse {
    set_beacon_device_disabled(req, state, session, id.into_inner(), true).await
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/beacon-devices/{id}/enable",
    tag = "admin",
    params(("id" = i64, Path, description = "Beacon device id")),
    responses(
        (status = 200, description = "Beacon device enabled", body = inline(ApiResponse<ToggleResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn enable_beacon_device(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    id: web::Path<i64>,
) -> HttpResponse {
    set_beacon_device_disabled(req, state, session, id.into_inner(), false).await
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/beacon-devices/{id}/tokens",
    tag = "admin",
    params(("id" = i64, Path, description = "Beacon device id")),
    responses(
        (status = 200, description = "Beacon device tokens", body = inline(ApiResponse<Vec<BeaconDeviceTokenResponse>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_beacon_device_tokens(
    state: web::Data<crate::runtime::AppState>,
    id: web::Path<i64>,
) -> HttpResponse {
    match beacon_repo::list_device_tokens(state.db_handles.reader(), id.into_inner()).await {
        Ok(tokens) => HttpResponse::Ok().json(ApiResponse::ok(
            tokens
                .into_iter()
                .map(BeaconDeviceTokenResponse::from)
                .collect::<Vec<_>>(),
        )),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/beacon-devices/{id}/tokens",
    tag = "admin",
    request_body = CreateBeaconDeviceTokenAdminRequest,
    params(("id" = i64, Path, description = "Beacon device id")),
    responses(
        (status = 201, description = "Beacon device token created", body = inline(ApiResponse<CreateBeaconDeviceTokenAdminResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn create_beacon_device_token(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    id: web::Path<i64>,
    body: web::Json<CreateBeaconDeviceTokenAdminRequest>,
) -> HttpResponse {
    let device_id = id.into_inner();

    match crate::services::beacon_service::create_device_token(
        state.get_ref(),
        device_id,
        crate::types::CreateBeaconDeviceTokenRequest {
            name: body.into_inner().name,
        },
    )
    .await
    {
        Ok(response) => {
            let audit_ctx = AuditContext::from_request(&req, session.user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "beacon_device_token.create",
                    entity_type: "beacon_device",
                    entity_id: Some(device_id),
                    entity_name: None,
                },
                None,
            )
            .await;
            HttpResponse::Created().json(ApiResponse::ok(CreateBeaconDeviceTokenAdminResponse {
                token: response.token,
            }))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/beacon-devices/{id}/tokens/{token_id}/revoke",
    tag = "admin",
    params(
        ("id" = i64, Path, description = "Beacon device id"),
        ("token_id" = i64, Path, description = "Beacon device token id")
    ),
    responses(
        (status = 200, description = "Beacon device token revoked", body = inline(ApiResponse<RevokeResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn revoke_beacon_device_token(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    path: web::Path<(i64, i64)>,
) -> HttpResponse {
    let (device_id, token_id) = path.into_inner();

    match beacon_repo::revoke_device_token(state.db_handles.writer(), device_id, token_id).await {
        Ok(revoked) => {
            if revoked {
                let audit_ctx = AuditContext::from_request(&req, session.user.id);
                crate::services::audit_service::record_admin_action_best_effort(
                    state.get_ref(),
                    AdminAuditLogInput {
                        ctx: &audit_ctx,
                        action: "beacon_device_token.revoke",
                        entity_type: "beacon_device_token",
                        entity_id: Some(token_id),
                        entity_name: None,
                    },
                    Some(serde_json::json!({ "device_id": device_id })),
                )
                .await;
            }
            HttpResponse::Ok().json(ApiResponse::ok(RevokeResponse { revoked }))
        }
        Err(error) => common::app_error(error),
    }
}

async fn set_beacon_device_disabled(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    device_id: i64,
    disabled: bool,
) -> HttpResponse {
    match beacon_repo::set_device_disabled(state.db_handles.writer(), device_id, disabled).await {
        Ok(changed) => {
            if changed {
                let audit_ctx = AuditContext::from_request(&req, session.user.id);
                crate::services::audit_service::record_admin_action_best_effort(
                    state.get_ref(),
                    AdminAuditLogInput {
                        ctx: &audit_ctx,
                        action: if disabled {
                            "beacon_device.disable"
                        } else {
                            "beacon_device.enable"
                        },
                        entity_type: "beacon_device",
                        entity_id: Some(device_id),
                        entity_name: None,
                    },
                    None,
                )
                .await;
            }
            HttpResponse::Ok().json(ApiResponse::ok(ToggleResponse { changed }))
        }
        Err(error) => common::app_error(error),
    }
}
