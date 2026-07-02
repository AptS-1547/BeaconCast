//! SeaORM entity for `activity_application_aliases`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "activity_application_aliases")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub application_id: i64,
    pub alias: String,
    pub normalized_alias: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
