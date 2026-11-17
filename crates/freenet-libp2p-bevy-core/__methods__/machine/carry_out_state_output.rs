use crate::replica_sync;
use crate::spectator;

pub fn carry_out_state_output<S: replica_sync::traits::ReplicatedState>(
    output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
    spectated: Option<&mut spectator::Spectated<S>>,
) {
    use spectator::state_machine::SpectateOutput;
    match output {
        SpectateOutput::Merge(bytes) => {
            if let Some(spectated) = spectated
                && let Ok(snapshot) = bincode::deserialize::<S::Snapshot>(bytes)
            {
                spectated.state.merge(snapshot);
                spectated.freshness = spectator::SpectateFreshness::Live;
            }
        }
        SpectateOutput::Apply { meta, op } => {
            if let Some(spectated) = spectated {
                spectated.state.apply(*meta, op);
            }
        }
        SpectateOutput::RepairFinished => {
            if let Some(spectated) = spectated {
                spectated.freshness = spectator::SpectateFreshness::Live;
            }
        }
        SpectateOutput::RequestSlice { .. } => {
            if let Some(spectated) = spectated {
                spectated.freshness = spectator::SpectateFreshness::Repairing;
            }
        }
        SpectateOutput::Subscribe(_)
        | SpectateOutput::Unsubscribe(_)
        | SpectateOutput::WatchZone(_)
        | SpectateOutput::UnwatchZone(_)
        | SpectateOutput::GetHistoryRecord(_)
        | SpectateOutput::Publish(_)
        | SpectateOutput::PublishLease { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use derive_more::Deref;

    use super::carry_out_state_output;
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;

    #[derive(Default, Resource, Deref)]
    struct MockState(Option<u8>);

    impl replica_sync::traits::ReplicatedState for MockState {
        type Op = u8;
        type Snapshot = u8;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &u8) {}

        fn snapshot(&self) -> u8 {
            0
        }

        fn merge(&mut self, snapshot: u8) {
            let Self(merged) = self;
            *merged = Some(snapshot);
        }

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> u8 {
            0
        }
    }

    fn spectated() -> spectator::Spectated<MockState> {
        spectator::Spectated {
            slot: spectator::SpectateSlot::Preview,
            zone: identity::ZoneName::from("zone"),
            state: MockState::default(),
            freshness: spectator::SpectateFreshness::Bootstrapping,
        }
    }

    #[test]
    fn test_usage() {
        let mut target = spectated();
        let bytes = bincode::serialize(&7_u8).unwrap();
        carry_out_state_output::<MockState>(
            &spectator::state_machine::SpectateOutput::Merge(bytes),
            Some(&mut target),
        );
        assert_eq!(*target.state, Some(7));
        assert_eq!(target.freshness, spectator::SpectateFreshness::Live);
    }

    #[test]
    fn a_requested_slice_marks_the_preview_repairing() {
        let mut target = spectated();
        carry_out_state_output::<MockState>(
            &spectator::state_machine::SpectateOutput::RequestSlice {
                from: identity::PeerId::from("holder"),
                index: replica_sync::ReplicaSliceIndex(0),
            },
            Some(&mut target),
        );
        assert_eq!(target.freshness, spectator::SpectateFreshness::Repairing);
    }
}
