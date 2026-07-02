//! SeaORM entity for `activity_actions`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "activity_actions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub action_key: String,
    pub label: String,
    pub status: String,
    pub category: String,
    pub public_label: String,
    pub message_template: String,
    pub enabled: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
