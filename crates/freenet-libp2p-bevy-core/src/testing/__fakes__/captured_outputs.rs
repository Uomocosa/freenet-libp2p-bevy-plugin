use bevy::prelude::Resource;

use crate::p2p;
use crate::replica_sync;

#[derive(Resource)]
pub struct CapturedOutputs<Op: Send + Sync + 'static> {
    pub commands: Vec<p2p::Libp2pCommand>,
    pub inputs: Vec<replica_sync::state_machine::Input<Op>>,
}

impl<Op: Send + Sync + 'static> Default for CapturedOutputs<Op> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            inputs: Vec::new(),
        }
    }
}
