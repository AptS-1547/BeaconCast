pub use beacon_cast_contract::{
    ActivityMetadata, AgentBeaconRequest, AgentCapabilitiesRequest, AgentCapabilitiesResponse,
    AgentCapability, AgentConfigResponse, AgentUsageSpan, AgentUsageSpansRequest,
    BeaconAcceptedResponse, UsageSpansAcceptedResponse,
};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiEnvelope<T> {
    pub code: String,
    pub msg: String,
    pub data: Option<T>,
}
