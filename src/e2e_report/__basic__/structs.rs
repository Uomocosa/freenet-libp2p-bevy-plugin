use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepTiming {
    pub raw: String,
    pub secs: f64,
    pub failure: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FreenetLayerResult {
    pub lobby_attached_after_secs: Option<f64>,
    pub freenet_peers_seen: usize,
}
