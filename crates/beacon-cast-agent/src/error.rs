pub type Result<T> = std::result::Result<T, AgentError>;

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("failed to read config file {path}: {source}")]
    ConfigRead {
        path: std::path::PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse config file {path}: {source}")]
    ConfigParse {
        path: std::path::PathBuf,
        source: toml::de::Error,
    },

    #[error("invalid config: {0}")]
    ConfigInvalid(String),

    #[error("invalid server url: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("failed to build HTTP client: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("BeaconCast server returned HTTP {status}: {message}")]
    RemoteStatus {
        status: reqwest::StatusCode,
        message: String,
    },

    #[error("source command failed: {0}")]
    SourceCommand(String),
}
