//! SeaORM entity for `beacon_device_tokens`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "beacon_device_tokens")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub device_id: i64,
    pub name: String,
    pub token_hash: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
