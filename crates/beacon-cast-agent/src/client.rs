use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};

use crate::config::AgentConfig;
use crate::dto::{
    AgentBeaconRequest, AgentCapabilitiesRequest, AgentCapabilitiesResponse, AgentConfigResponse,
    AgentUsageSpansRequest, ApiEnvelope, BeaconAcceptedResponse, UsageSpansAcceptedResponse,
};
use crate::error::{AgentError, Result};

#[derive(Clone)]
pub struct BeaconClient {
    base_url: url::Url,
    inner: reqwest::Client,
}

impl BeaconClient {
    pub fn new(config: &AgentConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {}", config.server.token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&bearer)
                .map_err(|error| AgentError::ConfigInvalid(error.to_string()))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&config.runtime.user_agent)
                .map_err(|error| AgentError::ConfigInvalid(error.to_string()))?,
        );

        let inner = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(config.request_timeout())
            .build()?;

        Ok(Self {
            base_url: config.server.url.clone(),
            inner,
        })
    }

    pub async fn agent_config(&self) -> Result<AgentConfigResponse> {
        self.get_api("/api/v1/beacon/agent/config").await
    }

    pub async fn put_capabilities(
        &self,
        input: &AgentCapabilitiesRequest,
    ) -> Result<AgentCapabilitiesResponse> {
        self.put_api("/api/v1/beacon/agent/capabilities", input)
            .await
    }

    pub async fn post_signal(&self, signal: &AgentBeaconRequest) -> Result<BeaconAcceptedResponse> {
        self.post_api("/api/v1/beacon/signals", signal).await
    }

    pub async fn post_usage_spans(
        &self,
        spans: &AgentUsageSpansRequest,
    ) -> Result<UsageSpansAcceptedResponse> {
        self.post_api("/api/v1/beacon/usage-spans", spans).await
    }

    async fn get_api<T>(&self, path: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.inner.get(self.url(path)?).send().await?;
        self.decode_response(response).await
    }

    async fn post_api<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let response = self.inner.post(self.url(path)?).json(body).send().await?;
        self.decode_response(response).await
    }

    async fn put_api<T, B>(&self, path: &str, body: &B) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let response = self.inner.put(self.url(path)?).json(body).send().await?;
        self.decode_response(response).await
    }

    async fn decode_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(AgentError::RemoteStatus {
                status,
                message: remote_message(&body),
            });
        }

        let envelope = serde_json::from_str::<ApiEnvelope<T>>(&body).map_err(|error| {
            AgentError::RemoteStatus {
                status,
                message: format!("invalid API envelope: {error}"),
            }
        })?;
        envelope.data.ok_or_else(|| AgentError::RemoteStatus {
            status,
            message: format!(
                "API response did not include data: code={}, msg={}",
                envelope.code, envelope.msg
            ),
        })
    }

    fn url(&self, path: &str) -> Result<url::Url> {
        Ok(self.base_url.join(path.trim_start_matches('/'))?)
    }
}

fn remote_message(body: &str) -> String {
    serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("msg")
                .and_then(|msg| msg.as_str())
                .map(str::to_string)
        })
        .filter(|message| !message.is_empty())
        .unwrap_or_else(|| body.trim().to_string())
}
