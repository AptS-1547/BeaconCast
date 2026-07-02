use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use url::Url;

use crate::error::{AgentError, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub server: ServerConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    pub source: SourceConfig,
    #[serde(default)]
    pub rules: Vec<ActivityRule>,
    #[serde(default)]
    pub idle_detector: IdleDetectorConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(deserialize_with = "deserialize_url")]
    pub url: Url,
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RuntimeConfig {
    #[serde(default = "default_poll_interval_seconds")]
    pub poll_interval_seconds: u64,
    #[serde(default = "default_request_timeout_seconds")]
    pub request_timeout_seconds: u64,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: default_poll_interval_seconds(),
            request_timeout_seconds: default_request_timeout_seconds(),
            log_level: default_log_level(),
            user_agent: default_user_agent(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceConfig {
    Manual(ManualSourceConfig),
    FrontmostApp(FrontmostAppSourceConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManualSourceConfig {
    #[serde(default)]
    pub project_key: Option<String>,
    #[serde(default)]
    pub project_label: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrontmostAppSourceConfig {
    #[serde(default = "default_source_name")]
    pub source_name: String,
    #[serde(default)]
    pub default: Option<ManualSourceConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActivityRule {
    pub app: String,
    #[serde(default)]
    pub project_key: Option<String>,
    #[serde(default)]
    pub project_label: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IdleDetectorConfig {
    pub enabled: bool,
    pub idle_after_seconds: u64,
}

impl Default for IdleDetectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_after_seconds: 600,
        }
    }
}

impl AgentConfig {
    pub async fn load(path: &Path) -> Result<Self> {
        let content =
            tokio::fs::read_to_string(path)
                .await
                .map_err(|source| AgentError::ConfigRead {
                    path: path.to_path_buf(),
                    source,
                })?;
        Self::from_toml(&content, path)
    }

    pub fn from_toml(content: &str, path: &Path) -> Result<Self> {
        let config = toml::from_str::<Self>(content).map_err(|source| AgentError::ConfigParse {
            path: PathBuf::from(path),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.runtime.poll_interval_seconds.max(1))
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.runtime.request_timeout_seconds.max(1))
    }

    fn validate(&self) -> Result<()> {
        if self.server.token.trim().is_empty() {
            return Err(AgentError::ConfigInvalid(
                "server.token must not be empty".to_string(),
            ));
        }
        if !matches!(self.server.url.scheme(), "http" | "https") {
            return Err(AgentError::ConfigInvalid(
                "server.url must use http or https".to_string(),
            ));
        }
        validate_log_level(&self.runtime.log_level)?;
        for rule in &self.rules {
            validate_public_label("rules.app", &rule.app)?;
        }
        Ok(())
    }
}

fn validate_public_label(field: &str, value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 120 {
        return Err(AgentError::ConfigInvalid(format!(
            "{field} must be 1-120 characters"
        )));
    }
    Ok(())
}

fn validate_log_level(value: &str) -> Result<()> {
    match value.trim().to_ascii_lowercase().as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => Ok(()),
        _ => Err(AgentError::ConfigInvalid(
            "runtime.log_level must be trace, debug, info, warn, or error".to_string(),
        )),
    }
}

fn default_poll_interval_seconds() -> u64 {
    30
}

fn default_request_timeout_seconds() -> u64 {
    10
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_user_agent() -> String {
    concat!("beacon-cast-agent/", env!("CARGO_PKG_VERSION")).to_string()
}

fn default_source_name() -> String {
    "frontmost_app".to_string()
}

fn default_confidence() -> f32 {
    0.8
}

fn deserialize_url<'de, D>(deserializer: D) -> std::result::Result<Url, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Url::parse(&value).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manual_config() {
        let config = AgentConfig::from_toml(
            r#"
                [server]
                url = "http://127.0.0.1:8748"
                token = "bcast_test"

                [runtime]
                log_level = "debug"

                [source]
                kind = "manual"
                project_key = "beacon_cast"
                project_label = "BeaconCast"
            "#,
            Path::new("agent.toml"),
        )
        .expect("manual config should parse");

        assert_eq!(config.server.url.as_str(), "http://127.0.0.1:8748/");
        assert_eq!(config.poll_interval(), Duration::from_secs(30));
        assert_eq!(config.runtime.log_level, "debug");
    }

    #[test]
    fn rejects_invalid_log_level() {
        let error = AgentConfig::from_toml(
            r#"
                [server]
                url = "http://127.0.0.1:8748"
                token = "bcast_test"

                [runtime]
                log_level = "verbose"

                [source]
                kind = "manual"
            "#,
            Path::new("agent.toml"),
        )
        .expect_err("invalid log level should be rejected");

        assert!(error.to_string().contains("runtime.log_level"));
    }
}
