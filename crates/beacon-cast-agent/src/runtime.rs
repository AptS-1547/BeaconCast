use crate::client::BeaconClient;
use crate::config::AgentConfig;
use crate::dto::{
    ActivityMetadata, AgentBeaconRequest, AgentCapabilitiesRequest, AgentConfigResponse,
    AgentUsageSpan, AgentUsageSpansRequest,
};
use crate::error::Result;
use crate::sources::{self, CollectionPolicy};
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

const ACTIVE_SPAN_CHECKPOINT_SECONDS: i64 = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Once,
    Forever,
    DryRun,
}

impl RunMode {
    pub fn from_args(once: bool, dry_run: bool) -> Self {
        if dry_run {
            Self::DryRun
        } else if once {
            Self::Once
        } else {
            Self::Forever
        }
    }
}

pub async fn run(config: AgentConfig, mode: RunMode) -> Result<()> {
    if mode == RunMode::DryRun {
        let report = sources::diagnose(&config, CollectionPolicy::default()).await?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| { crate::error::AgentError::SourceCommand(error.to_string()) })?
        );
        return Ok(());
    }

    let client = BeaconClient::new(&config)?;
    let mut policy = AgentRuntimePolicy::local(&config);
    let mut next_policy_refresh = Instant::now();
    let mut spans = UsageSpanAccumulator::default();

    report_capabilities(&config, &client).await;
    refresh_policy_if_due(&client, &mut policy, &mut next_policy_refresh).await;
    let signal = run_iteration(&config, &client, policy.collection).await?;
    spans.observe(&signal, Utc::now());
    if mode == RunMode::Once {
        return Ok(());
    }

    loop {
        tokio::select! {
            _ = tokio::time::sleep(policy.report_interval) => {
                refresh_policy_if_due(&client, &mut policy, &mut next_policy_refresh).await;
                match run_iteration(&config, &client, policy.collection).await {
                    Ok(signal) => {
                        spans.observe(&signal, Utc::now());
                        upload_pending_spans(&client, &mut spans).await;
                    }
                    Err(error) => {
                        tracing::warn!(%error, "failed to publish beacon signal");
                    }
                }
            }
            shutdown = shutdown_signal() => {
                shutdown?;
                tracing::info!("BeaconCast agent shutdown requested");
                spans.flush(Utc::now());
                upload_pending_spans(&client, &mut spans).await;
                return Ok(());
            }
        }
    }
}

async fn report_capabilities(config: &AgentConfig, client: &BeaconClient) {
    let capabilities = sources::capabilities(config);
    match client
        .put_capabilities(&AgentCapabilitiesRequest { capabilities })
        .await
    {
        Ok(response) => {
            tracing::info!(
                accepted = response.accepted,
                capabilities = ?response.capabilities,
                "reported agent capabilities"
            );
        }
        Err(error) => {
            tracing::warn!(%error, "failed to report agent capabilities");
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AgentRuntimePolicy {
    report_interval: Duration,
    config_poll_interval: Duration,
    collection: CollectionPolicy,
}

impl AgentRuntimePolicy {
    fn local(config: &AgentConfig) -> Self {
        Self {
            report_interval: config.poll_interval(),
            config_poll_interval: config.poll_interval(),
            collection: CollectionPolicy::default(),
        }
    }

    fn from_remote(remote: &AgentConfigResponse, fallback: Self) -> Self {
        Self {
            report_interval: seconds_or(remote.report_interval_seconds, fallback.report_interval),
            config_poll_interval: seconds_or(
                remote.poll_interval_seconds,
                fallback.config_poll_interval,
            ),
            collection: CollectionPolicy {
                include_app_label: remote.include_app_label,
            },
        }
    }
}

async fn refresh_policy_if_due(
    client: &BeaconClient,
    policy: &mut AgentRuntimePolicy,
    next_refresh: &mut Instant,
) {
    if Instant::now() < *next_refresh {
        return;
    }
    match client.agent_config().await {
        Ok(remote_config) => {
            *policy = AgentRuntimePolicy::from_remote(&remote_config, *policy);
            *next_refresh = Instant::now() + policy.config_poll_interval;
            tracing::info!(
                config_poll_interval_seconds = remote_config.poll_interval_seconds,
                report_interval_seconds = remote_config.report_interval_seconds,
                offline_after_seconds = remote_config.offline_after_seconds,
                idle_after_seconds = remote_config.idle_after_seconds,
                private_mode_enabled = remote_config.private_mode_enabled,
                include_app_label = remote_config.include_app_label,
                "loaded agent policy from BeaconCast server"
            );
        }
        Err(error) => {
            *next_refresh = Instant::now() + policy.config_poll_interval;
            tracing::warn!(%error, "failed to load remote agent policy; using current policy");
        }
    }
}

fn seconds_or(value: u64, fallback: Duration) -> Duration {
    if value == 0 {
        fallback
    } else {
        Duration::from_secs(value)
    }
}

async fn run_iteration(
    config: &AgentConfig,
    client: &BeaconClient,
    collection_policy: CollectionPolicy,
) -> Result<AgentBeaconRequest> {
    let signal = sources::collect(config, collection_policy).await?;
    let accepted = client.post_signal(&signal).await?;
    tracing::info!(
        event_id = accepted.event_id,
        accepted = accepted.accepted,
        source = %signal.source,
        app_label = ?signal.app_label,
        project_key = ?signal.project_key,
        idle = signal.idle,
        "published beacon signal"
    );
    Ok(signal)
}

async fn upload_pending_spans(client: &BeaconClient, spans: &mut UsageSpanAccumulator) {
    let pending = spans.take_pending();
    if pending.is_empty() {
        return;
    }
    let pending_count = pending.len();
    match client
        .post_usage_spans(&AgentUsageSpansRequest { spans: pending })
        .await
    {
        Ok(response) => {
            tracing::info!(
                accepted = response.accepted,
                spans_accepted = response.spans_accepted,
                "published usage spans"
            );
        }
        Err(error) => {
            tracing::warn!(%error, pending_count, "failed to publish usage spans");
            spans.restore_unconfirmed(pending_count);
        }
    }
}

#[derive(Debug, Default)]
struct UsageSpanAccumulator {
    active: Option<ActiveUsageSpan>,
    pending: Vec<AgentUsageSpan>,
    unconfirmed: Vec<AgentUsageSpan>,
}

impl UsageSpanAccumulator {
    fn observe(&mut self, signal: &AgentBeaconRequest, observed_at: DateTime<Utc>) {
        let key = UsageSpanKey::from_signal(signal);
        if let Some(active) = &mut self.active
            && active.key == key
        {
            active.last_seen_at = observed_at;
            if observed_at
                .signed_duration_since(active.started_at)
                .num_seconds()
                >= ACTIVE_SPAN_CHECKPOINT_SECONDS
            {
                self.flush(observed_at);
                self.active = Some(ActiveUsageSpan {
                    key,
                    started_at: observed_at,
                    last_seen_at: observed_at,
                    signal: signal.clone(),
                });
            }
            return;
        }

        self.flush(observed_at);
        self.active = Some(ActiveUsageSpan {
            key,
            started_at: observed_at,
            last_seen_at: observed_at,
            signal: signal.clone(),
        });
    }

    fn flush(&mut self, ended_at: DateTime<Utc>) {
        let Some(active) = self.active.take() else {
            return;
        };
        let ended_at = ended_at.max(active.last_seen_at);
        if ended_at <= active.started_at {
            return;
        }
        self.pending.push(AgentUsageSpan {
            started_at: active.started_at,
            ended_at,
            app_label: active.key.app_label,
            project_key: active.signal.project_key,
            project_label: active.signal.project_label,
            source: active.signal.source,
            idle: active.signal.idle,
            confidence: active.signal.confidence,
            metadata: active.signal.metadata,
        });
    }

    fn take_pending(&mut self) -> Vec<AgentUsageSpan> {
        self.unconfirmed = std::mem::take(&mut self.pending);
        self.unconfirmed.clone()
    }

    fn restore_unconfirmed(&mut self, count: usize) {
        if self.unconfirmed.len() != count {
            return;
        }
        let mut unconfirmed = std::mem::take(&mut self.unconfirmed);
        unconfirmed.append(&mut self.pending);
        self.pending = unconfirmed;
    }
}

#[derive(Debug, Clone)]
struct ActiveUsageSpan {
    key: UsageSpanKey,
    started_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
    signal: AgentBeaconRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UsageSpanKey {
    app_label: Option<String>,
    project_key: Option<String>,
    project_label: Option<String>,
    source: String,
    idle: bool,
    metadata: ActivityMetadata,
}

impl UsageSpanKey {
    fn from_signal(signal: &AgentBeaconRequest) -> Self {
        Self {
            app_label: signal.app_label.clone(),
            project_key: signal.project_key.clone(),
            project_label: signal.project_label.clone(),
            source: signal.source.clone(),
            idle: signal.idle,
            metadata: signal.metadata.clone(),
        }
    }
}

async fn shutdown_signal() -> Result<()> {
    tokio::signal::ctrl_c()
        .await
        .map_err(|error| crate::error::AgentError::SourceCommand(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::AgentBeaconRequest;

    #[test]
    fn run_mode_tracks_once_flag() {
        assert_eq!(RunMode::from_args(true, false), RunMode::Once);
        assert_eq!(RunMode::from_args(false, false), RunMode::Forever);
        assert_eq!(RunMode::from_args(false, true), RunMode::DryRun);
    }

    #[test]
    fn accumulator_closes_span_when_project_changes() {
        let mut accumulator = UsageSpanAccumulator::default();
        let start = DateTime::parse_from_rfc3339("2026-07-02T00:00:00Z")
            .expect("fixed timestamp")
            .with_timezone(&Utc);
        let mut signal = test_signal();

        accumulator.observe(&signal, start);
        accumulator.observe(&signal, start + chrono::Duration::seconds(30));
        assert!(accumulator.pending.is_empty());

        signal.project_key = Some("aster_drive".to_string());
        signal.project_label = Some("AsterDrive".to_string());
        accumulator.observe(&signal, start + chrono::Duration::seconds(60));

        assert_eq!(accumulator.pending.len(), 1);
        assert_eq!(
            accumulator.pending[0].project_key.as_deref(),
            Some("beacon_cast")
        );
        assert_eq!(accumulator.pending[0].app_label.as_deref(), Some("Code"));
    }

    #[test]
    fn accumulator_uses_reported_app_label_when_metadata_is_omitted() {
        let mut accumulator = UsageSpanAccumulator::default();
        let start = DateTime::parse_from_rfc3339("2026-07-02T00:00:00Z")
            .expect("fixed timestamp")
            .with_timezone(&Utc);
        let mut signal = test_signal();
        signal.metadata = ActivityMetadata::default();

        accumulator.observe(&signal, start);
        accumulator.flush(start + chrono::Duration::seconds(30));

        assert_eq!(accumulator.pending.len(), 1);
        assert_eq!(accumulator.pending[0].app_label.as_deref(), Some("Code"));
        assert!(accumulator.pending[0].metadata.is_empty());
    }

    #[test]
    fn accumulator_checkpoints_long_running_activity() {
        let mut accumulator = UsageSpanAccumulator::default();
        let start = DateTime::parse_from_rfc3339("2026-07-02T00:00:00Z")
            .expect("fixed timestamp")
            .with_timezone(&Utc);
        let signal = test_signal();

        accumulator.observe(&signal, start);
        accumulator.observe(
            &signal,
            start + chrono::Duration::seconds(ACTIVE_SPAN_CHECKPOINT_SECONDS),
        );

        assert_eq!(accumulator.pending.len(), 1);
        assert_eq!(accumulator.pending[0].source, "frontmost_app");
        assert!(accumulator.active.is_some());
    }

    fn test_signal() -> AgentBeaconRequest {
        AgentBeaconRequest {
            observed_at: None,
            app_label: Some("Code".to_string()),
            project_key: Some("beacon_cast".to_string()),
            project_label: Some("BeaconCast".to_string()),
            source: "frontmost_app".to_string(),
            idle: false,
            confidence: 0.9,
            metadata: ActivityMetadata::default(),
        }
    }
}
