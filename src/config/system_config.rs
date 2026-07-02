//! Runtime system configuration helpers.

use crate::config::definitions::CONFIG_REGISTRY;
use aster_forge_config::{ConfigDefinition, ConfigSource, StoredConfig};
use aster_forge_db::system_config;

pub fn get_definition(key: &str) -> Option<&'static ConfigDefinition> {
    CONFIG_REGISTRY.get(key)
}

pub fn apply_definition(mut config: system_config::Model) -> system_config::Model {
    if config.source != ConfigSource::System {
        return config;
    }

    let stored = CONFIG_REGISTRY.apply_definition(model_to_stored_config(&config));
    config.value_type = stored.value_type;
    config.requires_restart = stored.requires_restart;
    config.is_sensitive = stored.is_sensitive;
    config.visibility = stored.visibility;
    config.category = stored.category;
    config.description = stored.description;
    config
}

pub fn snapshot_from_models(
    configs: Vec<system_config::Model>,
) -> aster_forge_config::SyncConfigSnapshot {
    aster_forge_config::SyncConfigSnapshot::from_configs(
        configs
            .into_iter()
            .map(|config| CONFIG_REGISTRY.apply_definition(model_to_stored_config(&config)))
            .collect(),
    )
}

fn model_to_stored_config(config: &system_config::Model) -> StoredConfig {
    StoredConfig {
        id: config.id,
        key: config.key.clone(),
        value: config.value.clone(),
        value_type: config.value_type,
        requires_restart: config.requires_restart,
        is_sensitive: config.is_sensitive,
        source: config.source,
        visibility: config.visibility,
        category: config.category.clone(),
        description: config.description.clone(),
    }
}
