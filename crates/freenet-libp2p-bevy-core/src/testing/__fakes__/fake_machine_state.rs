use bevy::prelude::Resource;

use crate::replica_sync;

#[derive(Default, Resource)]
pub struct FakeMachineState {
    pub merged: Option<u8>,
    pub applied: Vec<(replica_sync::OpMeta, u8)>,
}

#[rustfmt::skip]
impl replica_sync::traits::ReplicatedState for FakeMachineState {
    type Op = u8;
    type Snapshot = u8;

    fn apply(&mut self, meta: replica_sync::OpMeta, op: &u8) { self.applied.push((meta, *op)); }

    fn snapshot(&self) -> u8 { self.merged.unwrap_or_default() }

    fn merge(&mut self, snapshot: u8) { self.merged = Some(snapshot); }

    fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes { Vec::new() }

    fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> u8 { self.merged.unwrap_or_default() }
}

#[cfg(test)]
mod tests {
    use crate::replica_sync;
    use crate::replica_sync::traits::ReplicatedState;

    use super::FakeMachineState;

    #[test]
    fn test_usage() {
        let mut state = FakeMachineState::default();
        state.merge(5);
        state.apply(
            replica_sync::OpMeta {
                lamport: 1,
                author: 1,
            },
            &7,
        );
        assert_eq!(state.merged, Some(5));
        assert_eq!(state.applied.len(), 1);
        assert_eq!(state.snapshot(), 5);
    }
}
