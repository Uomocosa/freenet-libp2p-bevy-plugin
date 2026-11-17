use std::collections::BTreeMap;
use std::marker::PhantomData;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;

use crate::p2p;
use crate::spectator;

#[derive(Resource, Default)]
pub struct HistoryBootstrap<Snap> {
    pub prefix: Option<p2p::HistoryKeyPrefix>,
    pub manifest: Option<p2p::HistoryManifest>,
    pub chunks: BTreeMap<p2p::ChunkIndex, Vec<u8>>,
    pub marker: PhantomData<Snap>,
}

#[atomic_delegates]
impl<Snap> HistoryBootstrap<Snap> {
    pub(crate) fn handle_history_record(
        &mut self,
        id: &p2p::ZoneHistoryRecordKind,
        value: Vec<u8>,
    ) -> spectator::BootstrapOutcome {
    }
    pub(crate) fn reset(&mut self) {}
}

// no test_usage necessary
