//! Admin authentication routes.

use actix_web::cookie::time::Duration as CookieDuration;
use actix_web::cookie::{Cookie, SameSite};
use actix_web::{HttpRequest, HttpResponse, web};

use crate::api::common;
use crate::api::dto::auth::{AdminLoginRequest, AdminSetupRequest};
use crate::api::middleware::auth::AdminSessionContext;
use crate::api::middleware::csrf::{ADMIN_CSRF_COOKIE, build_csrf_token};
use crate::api::request_auth::ADMIN_SESSION_COOKIE;
use crate::api::response::ApiResponse;
use crate::services::audit_service::{AdminAuditLogInput, AuditContext};

fn build_session_cookie(value: &str, max_age_secs: i64, secure: bool) -> Cookie<'static> {
    Cookie::build(ADMIN_SESSION_COOKIE.to_string(), value.to_string())
        .path("/".to_string())
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(CookieDuration::seconds(max_age_secs))
        .finish()
}

fn clear_session_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build(ADMIN_SESSION_COOKIE.to_string(), "")
        .path("/".to_string())
        .http_only(true)
        .secure(secure)
        .max_age(CookieDuration::ZERO)
        .finish()
}

fn build_csrf_cookie(value: &str, max_age_secs: i64, secure: bool) -> Cookie<'static> {
    Cookie::build(ADMIN_CSRF_COOKIE.to_string(), value.to_string())
        .path("/".to_string())
        .http_only(false)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(CookieDuration::seconds(max_age_secs))
        .finish()
}

fn clear_csrf_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build(ADMIN_CSRF_COOKIE.to_string(), "")
        .path("/".to_string())
        .http_only(false)
        .same_site(SameSite::Lax)
        .secure(secure)
        .max_age(CookieDuration::ZERO)
        .finish()
}

fn authenticated_response(
    state: &crate::runtime::AppState,
    issued: crate::services::admin_auth_service::IssuedAdminSession,
    status: actix_web::http::StatusCode,
) -> HttpResponse {
    let secure = state.config.auth.cookie_secure;
    let max_age = i64::try_from(issued.response.expires_in).unwrap_or(i64::MAX);
    let csrf_token = build_csrf_token();
    HttpResponse::build(status)
        .cookie(build_session_cookie(&issued.token, max_age, secure))
        .cookie(build_csrf_cookie(&csrf_token, max_age, secure))
        .json(ApiResponse::ok(issued.response))
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/auth/check",
    tag = "admin-auth",
    responses(
        (status = 200, description = "Admin setup state", body = inline(ApiResponse<crate::api::dto::auth::AdminAuthCheckResponse>)),
    )
)]
pub async fn check_auth(state: web::Data<crate::runtime::AppState>) -> HttpResponse {
    match crate::services::admin_auth_service::check(state.get_ref()).await {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/auth/setup",
    tag = "admin-auth",
    request_body = AdminSetupRequest,
    responses(
        (status = 201, description = "First admin account created", body = inline(ApiResponse<crate::api::dto::auth::AdminAuthResponse>)),
        (status = 400, description = "Already initialized or invalid input", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn setup_admin(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    body: web::Json<AdminSetupRequest>,
) -> HttpResponse {
    match crate::services::admin_auth_service::setup(state.get_ref(), body.into_inner()).await {
        Ok(issued) => {
            let user = &issued.response.user;
            let audit_ctx = AuditContext::from_request(&req, user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "admin.auth.setup",
                    entity_type: "admin_user",
                    entity_id: Some(user.id),
                    entity_name: Some(&user.username),
                },
                None,
            )
            .await;
            authenticated_response(
                state.get_ref(),
                issued,
                actix_web::http::StatusCode::CREATED,
            )
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/auth/login",
    tag = "admin-auth",
    request_body = AdminLoginRequest,
    responses(
        (status = 200, description = "Admin authenticated", body = inline(ApiResponse<crate::api::dto::auth::AdminAuthResponse>)),
        (status = 401, description = "Invalid credentials", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn login_admin(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    body: web::Json<AdminLoginRequest>,
) -> HttpResponse {
    match crate::services::admin_auth_service::login(state.get_ref(), body.into_inner()).await {
        Ok(issued) => {
            let user = &issued.response.user;
            let audit_ctx = AuditContext::from_request(&req, user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "admin.auth.login",
                    entity_type: "admin_user",
                    entity_id: Some(user.id),
                    entity_name: Some(&user.username),
                },
                None,
            )
            .await;
            authenticated_response(state.get_ref(), issued, actix_web::http::StatusCode::OK)
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/auth/logout",
    tag = "admin-auth",
    responses(
        (status = 200, description = "Admin session revoked", body = inline(ApiResponse<crate::api::dto::auth::AdminLogoutResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn logout_admin(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
) -> HttpResponse {
    match crate::services::admin_auth_service::logout(state.get_ref(), &session.token).await {
        Ok(response) => {
            let user = &session.user;
            let audit_ctx = AuditContext::from_request(&req, user.id);
            crate::services::audit_service::record_admin_action_best_effort(
                state.get_ref(),
                AdminAuditLogInput {
                    ctx: &audit_ctx,
                    action: "admin.auth.logout",
                    entity_type: "admin_user",
                    entity_id: Some(user.id),
                    entity_name: Some(&user.username),
                },
                None,
            )
            .await;
            let secure = state.config.auth.cookie_secure;
            HttpResponse::Ok()
                .cookie(clear_session_cookie(secure))
                .cookie(clear_csrf_cookie(secure))
                .json(ApiResponse::ok(response))
        }
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/me",
    tag = "admin",
    responses(
        (status = 200, description = "Current admin user", body = inline(ApiResponse<crate::api::dto::auth::AdminUserResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn me(session: web::ReqData<AdminSessionContext>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::ok(session.user.clone()))
}

#[aster_forge_api_docs_macros::path(
    get,
    path = "/api/v1/admin/sessions",
    tag = "admin",
    responses(
        (status = 200, description = "Current admin sessions", body = inline(ApiResponse<Vec<crate::api::dto::admin::AdminSessionResponse>>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn list_sessions(
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
) -> HttpResponse {
    match crate::services::admin_auth_service::list_sessions(
        state.get_ref(),
        session.user.id,
        &session.token,
    )
    .await
    {
        Ok(response) => HttpResponse::Ok().json(ApiResponse::ok(response)),
        Err(error) => common::app_error(error),
    }
}

#[aster_forge_api_docs_macros::path(
    post,
    path = "/api/v1/admin/sessions/{session_id}/revoke",
    tag = "admin",
    params(("session_id" = i64, Path, description = "Admin session id")),
    responses(
        (status = 200, description = "Admin session revoked", body = inline(ApiResponse<crate::api::dto::admin::RevokeResponse>)),
        (status = 401, description = "Missing or invalid admin token", body = inline(ApiResponse<crate::api::response::ApiEmptyData>)),
    )
)]
pub async fn revoke_session(
    req: HttpRequest,
    state: web::Data<crate::runtime::AppState>,
    session: web::ReqData<AdminSessionContext>,
    session_id: web::Path<i64>,
) -> HttpResponse {
    let session_id = session_id.into_inner();

    match crate::services::admin_auth_service::revoke_session(
        state.get_ref(),
        session.user.id,
        session_id,
    )
    .await
    {
        Ok(response) => {
            if response.revoked {
                let audit_ctx = AuditContext::from_request(&req, session.user.id);
                crate::services::audit_service::record_admin_action_best_effort(
                    state.get_ref(),
                    AdminAuditLogInput {
                        ctx: &audit_ctx,
                        action: "admin.session.revoke",
                        entity_type: "admin_session",
                        entity_id: Some(session_id),
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
