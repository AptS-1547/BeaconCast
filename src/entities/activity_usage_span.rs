//! SeaORM entity for `activity_usage_spans`.

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "activity_usage_spans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub device_id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
    pub app_label: Option<String>,
    pub project_key: Option<String>,
    pub project_label: Option<String>,
    pub idle: bool,
    pub source: String,
    pub confidence: f32,
    pub metadata_json: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
