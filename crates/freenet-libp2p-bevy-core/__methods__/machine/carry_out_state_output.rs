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
    use super::carry_out_state_output;
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut target = testing::spectated_state(testing::FakeMachineState::default());
        let bytes = bincode::serialize(&7_u8).unwrap();
        carry_out_state_output::<testing::FakeMachineState>(
            &spectator::state_machine::SpectateOutput::Merge(bytes),
            Some(&mut target),
        );
        assert_eq!(target.state.merged, Some(7));
        assert_eq!(target.freshness, spectator::SpectateFreshness::Live);
    }

    #[test]
    fn a_requested_slice_marks_the_preview_repairing() {
        let mut target = testing::spectated_state(testing::FakeMachineState::default());
        carry_out_state_output::<testing::FakeMachineState>(
            &spectator::state_machine::SpectateOutput::RequestSlice {
                from: identity::PeerId::from("holder"),
                index: replica_sync::ReplicaSliceIndex(0),
            },
            Some(&mut target),
        );
        assert_eq!(target.freshness, spectator::SpectateFreshness::Repairing);
    }
}
