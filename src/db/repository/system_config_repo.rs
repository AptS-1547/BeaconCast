//! Product-bound system config repository.

use crate::config::definitions::{CONFIG_REGISTRY, DEPRECATED_SYSTEM_CONFIG_KEYS};
use crate::errors::{AppError, Result};
use aster_forge_db::system_config::{self, SystemConfigDbBinding};
use sea_orm::ConnectionTrait;

static STORE: SystemConfigDbBinding =
    SystemConfigDbBinding::new(&CONFIG_REGISTRY, DEPRECATED_SYSTEM_CONFIG_KEYS);

fn map_store_result<T>(result: aster_forge_db::Result<T>) -> Result<T> {
    result.map_err(AppError::from)
}

pub async fn find_all<C: ConnectionTrait>(db: &C) -> Result<Vec<system_config::Model>> {
    map_store_result(STORE.find_all(db).await)
}

pub async fn ensure_defaults<C: ConnectionTrait>(db: &C) -> Result<usize> {
    map_store_result(STORE.ensure_defaults(db).await)
}

pub async fn upsert<C: ConnectionTrait>(
    db: &C,
    key: &str,
    value: &str,
    updated_by: Option<i64>,
) -> Result<system_config::Model> {
    map_store_result(
        STORE
            .upsert(
                db,
                aster_forge_db::system_config::SystemConfigUpsert {
                    key,
                    value,
                    visibility: None,
                    updated_by,
                },
            )
            .await,
    )
}
