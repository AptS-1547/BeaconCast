//! Admin authentication service.

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::api::dto::admin::{AdminSessionResponse, RevokeResponse};
use crate::api::dto::auth::{
    AdminAuthCheckResponse, AdminAuthResponse, AdminLoginRequest, AdminLogoutResponse,
    AdminSetupRequest, AdminUserResponse,
};
use crate::config::definitions::SECURITY_ADMIN_SESSION_TTL_SECONDS_KEY;
use crate::db::repository::{admin_auth_repo, system_config_repo};
use crate::entities::admin_user;
use crate::errors::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminClaims {
    pub sub: String,
    pub user_id: i64,
    pub username: String,
    pub jti: String,
    pub token_type: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Clone)]
pub struct IssuedAdminSession {
    pub token: String,
    pub response: AdminAuthResponse,
}

pub async fn check(state: &crate::runtime::AppState) -> Result<AdminAuthCheckResponse> {
    Ok(AdminAuthCheckResponse {
        initialized: admin_auth_repo::count_admin_users(state.db_handles.reader()).await? > 0,
    })
}

pub async fn setup(
    state: &crate::runtime::AppState,
    input: AdminSetupRequest,
) -> Result<IssuedAdminSession> {
    validate_admin_credentials(&input.username, &input.password)?;
    if input.display_name.trim().is_empty() {
        return Err(AppError::Validation(
            "display_name cannot be empty".to_string(),
        ));
    }
    if admin_auth_repo::count_admin_users(state.db_handles.writer()).await? > 0 {
        return Err(AppError::Validation(
            "system already initialized".to_string(),
        ));
    }

    let password_hash = aster_forge_crypto::hash_password(&input.password)?;
    let user = admin_auth_repo::create_admin_user(
        state.db_handles.writer(),
        input.username.trim().to_string(),
        password_hash,
        input.display_name.trim().to_string(),
    )
    .await?;
    issue_admin_token(state, user).await
}

pub async fn login(
    state: &crate::runtime::AppState,
    input: AdminLoginRequest,
) -> Result<IssuedAdminSession> {
    let Some(user) = admin_auth_repo::find_admin_user_by_username(
        state.db_handles.writer(),
        input.username.trim(),
    )
    .await?
    else {
        return Err(AppError::auth_credentials_failed("invalid credentials"));
    };

    if !aster_forge_crypto::verify_password(&input.password, &user.password_hash)? {
        return Err(AppError::auth_credentials_failed("invalid credentials"));
    }
    admin_auth_repo::touch_admin_login(state.db_handles.writer(), user.id).await?;
    issue_admin_token(state, user).await
}

pub async fn require_admin(
    state: &crate::runtime::AppState,
    token: &str,
) -> Result<AdminUserResponse> {
    let claims = verify_admin_token(token, &state.config.auth.jwt_secret)?;
    let token_hash = aster_forge_crypto::sha256_hex(token.as_bytes());
    let Some(session) = admin_auth_repo::find_active_admin_session_by_token_hash(
        state.db_handles.writer(),
        &token_hash,
    )
    .await?
    else {
        return Err(AppError::auth_token_invalid("invalid admin token"));
    };
    let Some(user) =
        admin_auth_repo::find_admin_user_by_id(state.db_handles.reader(), claims.user_id).await?
    else {
        return Err(AppError::auth_token_invalid("invalid admin token"));
    };
    admin_auth_repo::touch_admin_session(state.db_handles.writer(), session.id).await?;
    Ok(AdminUserResponse::from(user))
}

pub async fn logout(state: &crate::runtime::AppState, token: &str) -> Result<AdminLogoutResponse> {
    verify_admin_token(token, &state.config.auth.jwt_secret)?;
    let token_hash = aster_forge_crypto::sha256_hex(token.as_bytes());
    Ok(AdminLogoutResponse {
        revoked: admin_auth_repo::revoke_admin_session_by_token_hash(
            state.db_handles.writer(),
            &token_hash,
        )
        .await?,
    })
}

pub async fn list_sessions(
    state: &crate::runtime::AppState,
    user_id: i64,
    current_token: &str,
) -> Result<Vec<AdminSessionResponse>> {
    let current_hash = aster_forge_crypto::sha256_hex(current_token.as_bytes());
    let sessions = admin_auth_repo::list_admin_sessions(state.db_handles.reader(), user_id).await?;
    Ok(sessions
        .into_iter()
        .map(|session| AdminSessionResponse {
            id: session.id,
            user_id: session.user_id,
            expires_at: session.expires_at,
            revoked_at: session.revoked_at,
            created_at: session.created_at,
            last_seen_at: session.last_seen_at,
            current: session.token_hash == current_hash,
        })
        .collect())
}

pub async fn revoke_session(
    state: &crate::runtime::AppState,
    user_id: i64,
    session_id: i64,
) -> Result<RevokeResponse> {
    Ok(RevokeResponse {
        revoked: admin_auth_repo::revoke_admin_session_by_id(
            state.db_handles.writer(),
            user_id,
            session_id,
        )
        .await?,
    })
}

fn validate_admin_credentials(username: &str, password: &str) -> Result<()> {
    let username = username.trim();
    if username.len() < 3 || username.len() > 64 {
        return Err(AppError::Validation(
            "username must be between 3 and 64 characters".to_string(),
        ));
    }
    if password.len() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}

async fn issue_admin_token(
    state: &crate::runtime::AppState,
    user: admin_user::Model,
) -> Result<IssuedAdminSession> {
    let ttl_seconds = admin_session_ttl_seconds(state).await?;
    let now = Utc::now();
    let expires_at = now
        .checked_add_signed(Duration::seconds(i64::try_from(ttl_seconds).map_err(
            |_| AppError::Validation("admin session ttl is too large".to_string()),
        )?))
        .ok_or_else(|| AppError::Validation("admin session ttl overflow".to_string()))?;
    let token = create_admin_token(&user, now, ttl_seconds, &state.config.auth.jwt_secret)?;
    admin_auth_repo::create_admin_session(
        state.db_handles.writer(),
        user.id,
        aster_forge_crypto::sha256_hex(token.as_bytes()),
        expires_at,
    )
    .await?;
    Ok(IssuedAdminSession {
        token,
        response: AdminAuthResponse {
            expires_in: ttl_seconds,
            user: AdminUserResponse::from(user),
        },
    })
}

async fn admin_session_ttl_seconds(state: &crate::runtime::AppState) -> Result<u64> {
    let configs = system_config_repo::find_all(state.db_handles.reader()).await?;
    let snapshot = aster_forge_config::SyncConfigSnapshot::from_configs(configs);
    Ok(snapshot.get_u64_or(SECURITY_ADMIN_SESSION_TTL_SECONDS_KEY, 86_400))
}

fn create_admin_token(
    user: &admin_user::Model,
    now: chrono::DateTime<Utc>,
    ttl_seconds: u64,
    secret: &str,
) -> Result<String> {
    let iat = usize::try_from(now.timestamp())
        .map_err(|_| AppError::Validation("jwt issued-at is invalid".to_string()))?;
    let exp = iat
        .checked_add(
            usize::try_from(ttl_seconds)
                .map_err(|_| AppError::Validation("jwt ttl is too large".to_string()))?,
        )
        .ok_or_else(|| AppError::Validation("jwt exp overflow".to_string()))?;
    let claims = AdminClaims {
        sub: user.id.to_string(),
        user_id: user.id,
        username: user.username.clone(),
        jti: uuid::Uuid::new_v4().to_string(),
        token_type: "admin_access".to_string(),
        exp,
        iat,
    };
    Ok(encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}

pub fn verify_admin_token(token: &str, secret: &str) -> Result<AdminClaims> {
    let claims = decode::<AdminClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::auth_token_invalid("invalid admin token"))?;
    if claims.token_type != "admin_access" {
        return Err(AppError::auth_token_invalid("invalid admin token"));
    }
    Ok(claims)
}

impl From<admin_user::Model> for AdminUserResponse {
    fn from(value: admin_user::Model) -> Self {
        Self {
            id: value.id,
            username: value.username,
            display_name: value.display_name,
        }
    }
}
