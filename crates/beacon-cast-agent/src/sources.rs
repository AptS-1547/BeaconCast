use serde::Serialize;
use tokio::process::Command;

use crate::config::{ActivityRule, AgentConfig, ManualSourceConfig, SourceConfig};
use crate::dto::{ActivityMetadata, AgentBeaconRequest, AgentCapability};
use crate::error::Result;

#[derive(Debug, Clone, Copy)]
pub struct CollectionPolicy {
    pub include_app_label: bool,
}

impl Default for CollectionPolicy {
    fn default() -> Self {
        Self {
            include_app_label: true,
        }
    }
}

pub async fn collect(config: &AgentConfig, policy: CollectionPolicy) -> Result<AgentBeaconRequest> {
    let mut signal = match &config.source {
        SourceConfig::Manual(source) => manual_signal(source, "manual"),
        SourceConfig::FrontmostApp(source) => {
            let context = match frontmost_app_context().await {
                Ok(context) => Some(context),
                Err(error) => {
                    tracing::warn!(%error, "failed to collect frontmost app context");
                    None
                }
            };
            if let Some(app) = context.as_ref().map(|context| context.app.as_str()) {
                if let Some(rule) = find_rule(&config.rules, app) {
                    rule_signal(rule, &source.source_name, app, policy)
                } else if let Some(default) = source.default.as_ref() {
                    let mut signal = manual_signal(default, &source.source_name);
                    signal.app_label = policy
                        .include_app_label
                        .then(|| {
                            context
                                .as_ref()
                                .and_then(|context| safe_app_label(&context.app))
                        })
                        .flatten();
                    signal
                } else {
                    unknown_signal(
                        &source.source_name,
                        context.as_ref().map(|context| context.app.as_str()),
                        policy,
                    )
                }
            } else if let Some(default) = source.default.as_ref() {
                manual_signal(default, &source.source_name)
            } else {
                unknown_signal(&source.source_name, None, policy)
            }
        }
    };
    if idle_active(config).await? {
        apply_idle(&mut signal);
    }
    Ok(signal)
}

pub async fn diagnose(
    config: &AgentConfig,
    policy: CollectionPolicy,
) -> Result<CollectionDiagnosis> {
    let context = match &config.source {
        SourceConfig::FrontmostApp(_) => frontmost_app_context().await.ok(),
        SourceConfig::Manual(_) => None,
    };
    let signal = collect(config, policy).await?;
    Ok(CollectionDiagnosis {
        frontmost_app: context.as_ref().map(|context| context.app.clone()),
        window_title: context.as_ref().map(|context| context.window_title.clone()),
        window_title_empty: context
            .as_ref()
            .map(|context| context.window_title.trim().is_empty())
            .unwrap_or(true),
        app_label: signal.app_label.clone(),
        signal,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct CollectionDiagnosis {
    pub frontmost_app: Option<String>,
    pub window_title: Option<String>,
    pub window_title_empty: bool,
    pub app_label: Option<String>,
    pub signal: AgentBeaconRequest,
}

fn active_window_context() -> Result<FrontmostAppContext> {
    let window = x_win::get_active_window()
        .map_err(|error| crate::error::AgentError::SourceCommand(error.to_string()))?;
    let app = active_window_app_label(&window.info.name, &window.info.exec_name);
    let Some(app) = app else {
        return Err(crate::error::AgentError::SourceCommand(
            "active window provider returned no app name".to_string(),
        ));
    };

    Ok(FrontmostAppContext {
        app: app.to_string(),
        window_title: window.title,
    })
}

fn active_window_app_label(name: &str, exec_name: &str) -> Option<String> {
    first_non_empty([name, exec_name])
        .and_then(normalize_app_label)
        .and_then(|label| safe_app_label(&label))
}

fn first_non_empty<'a>(values: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    values
        .into_iter()
        .map(str::trim)
        .find(|value| !value.is_empty())
}

fn normalize_app_label(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_bundle = trimmed.strip_suffix(".app").unwrap_or(trimmed);
    let without_windows_exe = without_bundle
        .strip_suffix(".exe")
        .or_else(|| without_bundle.strip_suffix(".EXE"))
        .unwrap_or(without_bundle);
    Some(without_windows_exe.to_string())
}

pub fn capabilities(config: &AgentConfig) -> Vec<AgentCapability> {
    let mut capabilities = Vec::new();
    match &config.source {
        SourceConfig::Manual(_) => capabilities.push(AgentCapability::ManualSignal),
        SourceConfig::FrontmostApp(_) => capabilities.push(AgentCapability::FrontmostApp),
    }
    capabilities
}

fn find_rule<'a>(rules: &'a [ActivityRule], app: &str) -> Option<&'a ActivityRule> {
    rules.iter().find(|rule| rule.app.eq_ignore_ascii_case(app))
}

fn manual_signal(source: &ManualSourceConfig, source_name: &str) -> AgentBeaconRequest {
    AgentBeaconRequest {
        observed_at: None,
        app_label: None,
        project_key: source.project_key.clone(),
        project_label: source.project_label.clone(),
        source: source_name.to_string(),
        idle: false,
        confidence: source.confidence,
        metadata: ActivityMetadata::default(),
    }
}

fn rule_signal(
    rule: &ActivityRule,
    source_name: &str,
    app: &str,
    policy: CollectionPolicy,
) -> AgentBeaconRequest {
    AgentBeaconRequest {
        observed_at: None,
        app_label: policy
            .include_app_label
            .then(|| safe_app_label(app))
            .flatten(),
        project_key: rule.project_key.clone(),
        project_label: rule.project_label.clone(),
        source: source_name.to_string(),
        idle: false,
        confidence: rule.confidence,
        metadata: ActivityMetadata::default(),
    }
}

fn unknown_signal(
    source_name: &str,
    app: Option<&str>,
    policy: CollectionPolicy,
) -> AgentBeaconRequest {
    AgentBeaconRequest {
        observed_at: None,
        app_label: policy
            .include_app_label
            .then(|| app.and_then(safe_app_label))
            .flatten(),
        project_key: None,
        project_label: None,
        source: source_name.to_string(),
        idle: false,
        confidence: 0.3,
        metadata: ActivityMetadata::default(),
    }
}

fn safe_app_label(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.len() > 128
        || !trimmed.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(ch, ' ' | '-' | '_' | '.' | '/' | ':' | '#' | '(' | ')')
        })
    {
        return None;
    }
    Some(trimmed.to_string())
}

#[derive(Debug, Clone)]
struct FrontmostAppContext {
    app: String,
    window_title: String,
}

#[cfg(target_os = "macos")]
async fn frontmost_app_context() -> Result<FrontmostAppContext> {
    match active_window_context() {
        Ok(context)
            if !context.app.trim().is_empty() && !context.window_title.trim().is_empty() =>
        {
            return Ok(context);
        }
        Ok(context) => {
            tracing::debug!(
                app = %context.app,
                window_title_empty = context.window_title.trim().is_empty(),
                "active window provider did not include a usable title; trying System Events"
            );
        }
        Err(error) => {
            tracing::debug!(%error, "active window provider failed; trying System Events");
        }
    }

    let native_app = macos_frontmost_app_name();
    match frontmost_app_context_via_system_events().await {
        Ok(mut context) => {
            if context.app.trim().is_empty()
                && let Some(app) = native_app
            {
                context.app = app;
            }
            Ok(context)
        }
        Err(error) => {
            if let Some(app) = native_app {
                tracing::debug!(%error, "falling back to NSWorkspace frontmost app without window title");
                Ok(FrontmostAppContext {
                    app,
                    window_title: String::new(),
                })
            } else {
                Err(error)
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_frontmost_app_name() -> Option<String> {
    use objc2_app_kit::NSWorkspace;

    let workspace = NSWorkspace::sharedWorkspace();
    let app = workspace.frontmostApplication()?;
    let name = app.localizedName()?;
    safe_app_label(&name.to_string())
}

#[cfg(target_os = "macos")]
async fn frontmost_app_context_via_system_events() -> Result<FrontmostAppContext> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(
            r#"tell application "System Events"
set frontApp to first application process whose frontmost is true
set appName to name of frontApp
set windowTitle to ""
try
    set windowTitle to name of front window of frontApp
end try
return appName & linefeed & windowTitle
end tell"#,
        )
        .output()
        .await
        .map_err(|error| crate::error::AgentError::SourceCommand(error.to_string()))?;

    if !output.status.success() {
        return Err(crate::error::AgentError::SourceCommand(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    Ok(FrontmostAppContext {
        app: lines.next().unwrap_or_default().trim().to_string(),
        window_title: lines.next().unwrap_or_default().trim().to_string(),
    })
}

#[cfg(not(target_os = "macos"))]
async fn frontmost_app_context() -> Result<FrontmostAppContext> {
    active_window_context()
}

async fn idle_active(config: &AgentConfig) -> Result<bool> {
    if !config.idle_detector.enabled {
        return Ok(false);
    }
    let Some(seconds) = platform_idle_seconds().await? else {
        return Ok(false);
    };
    Ok(seconds >= config.idle_detector.idle_after_seconds)
}

fn apply_idle(signal: &mut AgentBeaconRequest) {
    signal.idle = true;
    signal.confidence = 0.95;
}

#[cfg(target_os = "macos")]
async fn platform_idle_seconds() -> Result<Option<u64>> {
    let output = Command::new("ioreg")
        .arg("-c")
        .arg("IOHIDSystem")
        .output()
        .await
        .map_err(|error| crate::error::AgentError::SourceCommand(error.to_string()))?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let Some(line) = stdout.lines().find(|line| line.contains("HIDIdleTime")) else {
        return Ok(None);
    };
    let Some(raw) = line
        .split('=')
        .nth(1)
        .and_then(|value| value.trim().parse::<u64>().ok())
    else {
        return Ok(None);
    };
    Ok(Some(raw / 1_000_000_000))
}

#[cfg(not(target_os = "macos"))]
async fn platform_idle_seconds() -> Result<Option<u64>> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ActivityRule, FrontmostAppSourceConfig, IdleDetectorConfig};

    #[test]
    fn rule_signal_uses_sanitized_config_labels() {
        let rule = ActivityRule {
            app: "Code".to_string(),
            project_key: Some("beacon_cast".to_string()),
            project_label: Some("BeaconCast".to_string()),
            confidence: 0.9,
        };

        let signal = rule_signal(&rule, "frontmost_app", "Code", CollectionPolicy::default());

        assert_eq!(signal.app_label.as_deref(), Some("Code"));
        assert_eq!(signal.project_key.as_deref(), Some("beacon_cast"));
        assert!(signal.metadata.is_empty());
    }

    #[test]
    fn rule_signal_can_omit_app_label() {
        let rule = ActivityRule {
            app: "Code".to_string(),
            project_key: None,
            project_label: None,
            confidence: 0.9,
        };

        let signal = rule_signal(
            &rule,
            "frontmost_app",
            "Code",
            CollectionPolicy {
                include_app_label: false,
            },
        );

        assert!(signal.app_label.is_none());
        assert!(signal.metadata.is_empty());
    }

    #[test]
    fn capabilities_are_derived_from_config() {
        let config = AgentConfig {
            server: crate::config::ServerConfig {
                url: url::Url::parse("http://127.0.0.1:8748").unwrap(),
                token: "bcast_test".to_string(),
            },
            runtime: crate::config::RuntimeConfig::default(),
            source: SourceConfig::FrontmostApp(FrontmostAppSourceConfig {
                source_name: "frontmost_app".to_string(),
                default: None,
            }),
            rules: Vec::new(),
            idle_detector: IdleDetectorConfig {
                enabled: false,
                idle_after_seconds: 600,
            },
        };

        assert_eq!(capabilities(&config), vec![AgentCapability::FrontmostApp]);
    }

    #[test]
    fn active_window_app_label_is_cross_platform_safe() {
        assert_eq!(
            active_window_app_label("Terminal", "Terminal.app").as_deref(),
            Some("Terminal")
        );
        assert_eq!(
            active_window_app_label("", "Code.exe").as_deref(),
            Some("Code")
        );
        assert_eq!(
            active_window_app_label("", "firefox").as_deref(),
            Some("firefox")
        );
    }
}
