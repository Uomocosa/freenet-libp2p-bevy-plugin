use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::e2e_report;
use crate::identity;
use crate::p2p;

#[derive(Resource, Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PeerRunResult {
    pub username: String,
    pub canvas_hash: Option<u64>,
    pub tiles: usize,
    pub members_seen: BTreeSet<identity::PeerId>,
    pub freenet: e2e_report::FreenetLayerResult,
    pub connection_kinds: BTreeMap<identity::PeerId, Vec<p2p::ConnectionKind>>,
    pub steps: Vec<e2e_report::StepTiming>,
    pub error: Option<String>,
    pub exit_code: u8,
}
