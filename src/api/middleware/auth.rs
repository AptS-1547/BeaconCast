//! Authentication middleware for admin browser sessions and agent bearer tokens.

use std::rc::Rc;

use actix_web::{
    Error, HttpMessage,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    web,
};
use futures::future::{LocalBoxFuture, Ready, ok};

use crate::api::{common, request_auth};
use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct AdminSessionContext {
    pub user: crate::api::dto::auth::AdminUserResponse,
    pub token: String,
}

#[derive(Debug, Clone)]
pub struct AgentBearerToken(pub String);

pub struct AdminSessionAuth;

impl<S, B> Transform<S, ServiceRequest> for AdminSessionAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminSessionAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminSessionAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct AdminSessionAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AdminSessionAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let Some(token) = request_auth::admin_session_cookie(req.request()) else {
                return Ok(error_response(
                    req,
                    AppError::auth_token_missing("missing admin session"),
                ));
            };

            if let Err(error) =
                crate::api::middleware::csrf::ensure_admin_cookie_write_allowed(req.request())
            {
                return Ok(error_response(req, error));
            }

            let Some(state) = req.app_data::<web::Data<crate::runtime::AppState>>() else {
                return Ok(error_response(
                    req,
                    AppError::Runtime("AppState not found".to_string()),
                ));
            };

            match crate::services::admin_auth_service::require_admin(state.get_ref(), &token).await
            {
                Ok(user) => {
                    req.extensions_mut()
                        .insert(AdminSessionContext { user, token });
                    svc.call(req).await.map(ServiceResponse::map_into_left_body)
                }
                Err(error) => Ok(error_response(req, error)),
            }
        })
    }
}

pub struct AgentBearerAuth;

impl<S, B> Transform<S, ServiceRequest> for AgentBearerAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AgentBearerAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AgentBearerAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct AgentBearerAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AgentBearerAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let Some(token) = request_auth::bearer_token(req.request()) else {
                return Ok(error_response(
                    req,
                    AppError::auth_token_missing("missing bearer token"),
                ));
            };

            req.extensions_mut()
                .insert(AgentBearerToken(token.to_string()));
            svc.call(req).await.map(ServiceResponse::map_into_left_body)
        })
    }
}

fn error_response<B>(req: ServiceRequest, error: AppError) -> ServiceResponse<EitherBody<B>> {
    req.into_response(common::app_error(error).map_into_right_body())
}
