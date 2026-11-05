use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;
use freenet_libp2p_bevy_plugin::{identity, p2p};
use serde::{Deserialize, Serialize};

use super::freenet_layer_result::FreenetLayerResult;
use super::step_timing::StepTiming;

#[derive(Resource, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PeerRunResult {
    pub username: String,
    pub canvas_hash: Option<u64>,
    pub tiles: usize,
    pub members_seen: BTreeSet<identity::PeerId>,
    pub freenet: FreenetLayerResult,
    pub connection_kinds: BTreeMap<identity::PeerId, Vec<p2p::ConnectionKind>>,
    pub steps: Vec<StepTiming>,
    pub error: Option<String>,
    pub exit_code: u8,
}
