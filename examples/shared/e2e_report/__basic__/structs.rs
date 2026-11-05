use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FreenetLayerResult {
    pub lobby_fetched_after_secs: Option<f64>,
    pub freenet_peers_seen: usize,
}
