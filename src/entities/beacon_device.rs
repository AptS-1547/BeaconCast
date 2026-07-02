//! SeaORM entity for `beacon_devices`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "beacon_devices")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub device_key: String,
    pub display_name: String,
    pub kind: String,
    pub priority: i32,
    pub capabilities_json: String,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
