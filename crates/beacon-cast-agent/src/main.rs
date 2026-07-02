//! BeaconCast local agent entrypoint.

mod client;
mod config;
mod dto;
mod error;
mod runtime;
mod sources;

use clap::Parser;
use error::Result;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Collects trusted local activity signals for BeaconCast"
)]
struct Args {
    /// Agent configuration file.
    #[arg(long, env = "BEACON_CAST_AGENT_CONFIG", default_value = "agent.toml")]
    config: std::path::PathBuf,

    /// Send one beacon and exit instead of running forever.
    #[arg(long)]
    once: bool,

    /// Print the sanitized beacon payload without contacting the server.
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = config::AgentConfig::load(&args.config).await?;
    let default_filter = config.runtime.log_level.trim();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| default_filter.into()),
        )
        .init();

    tracing::info!(
        config = %args.config.display(),
        server_url = %config.server.url,
        "BeaconCast agent starting"
    );

    runtime::run(config, runtime::RunMode::from_args(args.once, args.dry_run)).await
}
