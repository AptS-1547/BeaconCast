//! Admin authentication repository helpers.

use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};

use crate::entities::{admin_session, admin_user};
use crate::errors::Result;

pub async fn count_admin_users<C: ConnectionTrait>(db: &C) -> Result<u64> {
    Ok(admin_user::Entity::find().count(db).await?)
}

pub async fn create_admin_user<C: ConnectionTrait>(
    db: &C,
    username: String,
    password_hash: String,
    display_name: String,
) -> Result<admin_user::Model> {
    let now = Utc::now();
    Ok(admin_user::ActiveModel {
        username: Set(username),
        password_hash: Set(password_hash),
        display_name: Set(display_name),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_admin_user_by_username<C: ConnectionTrait>(
    db: &C,
    username: &str,
) -> Result<Option<admin_user::Model>> {
    Ok(admin_user::Entity::find()
        .filter(admin_user::Column::Username.eq(username))
        .filter(admin_user::Column::DisabledAt.is_null())
        .one(db)
        .await?)
}

pub async fn find_admin_user_by_id<C: ConnectionTrait>(
    db: &C,
    id: i64,
) -> Result<Option<admin_user::Model>> {
    Ok(admin_user::Entity::find_by_id(id)
        .filter(admin_user::Column::DisabledAt.is_null())
        .one(db)
        .await?)
}

pub async fn touch_admin_login<C: ConnectionTrait>(db: &C, user_id: i64) -> Result<()> {
    let now = Utc::now();
    admin_user::Entity::update_many()
        .col_expr(
            admin_user::Column::LastLoginAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .col_expr(
            admin_user::Column::UpdatedAt,
            sea_orm::sea_query::Expr::value(now),
        )
        .filter(admin_user::Column::Id.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn create_admin_session<C: ConnectionTrait>(
    db: &C,
    user_id: i64,
    token_hash: String,
    expires_at: chrono::DateTime<Utc>,
) -> Result<admin_session::Model> {
    let now = Utc::now();
    Ok(admin_session::ActiveModel {
        user_id: Set(user_id),
        token_hash: Set(token_hash),
        expires_at: Set(expires_at),
        created_at: Set(now),
        last_seen_at: Set(Some(now)),
        ..Default::default()
    }
    .insert(db)
    .await?)
}

pub async fn find_active_admin_session_by_token_hash<C: ConnectionTrait>(
    db: &C,
    token_hash: &str,
) -> Result<Option<admin_session::Model>> {
    Ok(admin_session::Entity::find()
        .filter(admin_session::Column::TokenHash.eq(token_hash))
        .filter(admin_session::Column::RevokedAt.is_null())
        .filter(admin_session::Column::ExpiresAt.gt(Utc::now()))
        .one(db)
        .await?)
}

pub async fn touch_admin_session<C: ConnectionTrait>(db: &C, session_id: i64) -> Result<()> {
    admin_session::Entity::update_many()
        .col_expr(
            admin_session::Column::LastSeenAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(admin_session::Column::Id.eq(session_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn revoke_admin_session_by_token_hash<C: ConnectionTrait>(
    db: &C,
    token_hash: &str,
) -> Result<bool> {
    let result = admin_session::Entity::update_many()
        .col_expr(
            admin_session::Column::RevokedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(admin_session::Column::TokenHash.eq(token_hash))
        .filter(admin_session::Column::RevokedAt.is_null())
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}

pub async fn list_admin_sessions<C: ConnectionTrait>(
    db: &C,
    user_id: i64,
) -> Result<Vec<admin_session::Model>> {
    Ok(admin_session::Entity::find()
        .filter(admin_session::Column::UserId.eq(user_id))
        .order_by_desc(admin_session::Column::CreatedAt)
        .all(db)
        .await?)
}

pub async fn revoke_admin_session_by_id<C: ConnectionTrait>(
    db: &C,
    user_id: i64,
    session_id: i64,
) -> Result<bool> {
    let result = admin_session::Entity::update_many()
        .col_expr(
            admin_session::Column::RevokedAt,
            sea_orm::sea_query::Expr::value(Utc::now()),
        )
        .filter(admin_session::Column::UserId.eq(user_id))
        .filter(admin_session::Column::Id.eq(session_id))
        .filter(admin_session::Column::RevokedAt.is_null())
        .exec(db)
        .await?;
    Ok(result.rows_affected > 0)
}
