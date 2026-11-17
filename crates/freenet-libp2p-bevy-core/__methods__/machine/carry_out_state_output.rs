use bevy::prelude::Commands;

use crate::replica_sync;
use crate::spectator;

pub fn carry_out_state_output<S: replica_sync::traits::ReplicatedState>(
    output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
    spectated: Option<&mut spectator::Spectated<S>>,
    count: &mut spectator::SpectatorCount,
    commands: &mut Commands,
    bootstrap: &mut spectator::HistoryBootstrap<Vec<u8>>,
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
        SpectateOutput::Clear => {
            commands.remove_resource::<spectator::Spectated<S>>();
            bootstrap.reset();
        }
        SpectateOutput::RepairFinished => {
            if let Some(spectated) = spectated {
                spectated.freshness = spectator::SpectateFreshness::Live;
            }
        }
        SpectateOutput::SpectatorCountChanged(spectators) => {
            **count = *spectators;
        }
        SpectateOutput::Subscribe(_)
        | SpectateOutput::Unsubscribe(_)
        | SpectateOutput::WatchZone(_)
        | SpectateOutput::UnwatchZone(_)
        | SpectateOutput::GetHistoryRecord(_)
        | SpectateOutput::RequestSlice { .. }
        | SpectateOutput::Publish(_)
        | SpectateOutput::PublishLease { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;
    use derive_more::Deref;

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
            zone: identity::ZoneName::from("zone"),
            state: MockState::default(),
            freshness: spectator::SpectateFreshness::Bootstrapping,
        }
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(spectator::SpectatorCount::default());
        app.init_resource::<spectator::HistoryBootstrap<Vec<u8>>>();
        app.world_mut()
            .run_system_once_with(
                |input: In<spectator::state_machine::SpectateOutput<u8, Vec<u8>>>,
                 mut count: ResMut<spectator::SpectatorCount>,
                 mut commands: Commands,
                 mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>| {
                    let In(output) = input;
                    spectator::Machine::<u8, Vec<u8>>::carry_out_state_output::<MockState>(
                        &output,
                        None,
                        &mut count,
                        &mut commands,
                        &mut bootstrap,
                    );
                },
                spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::SpectatorCountChanged(3),
            )
            .unwrap();
        assert_eq!(**app.world().resource::<spectator::SpectatorCount>(), 3);
    }

    #[test]
    fn merge_updates_state_and_marks_live() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(spectator::SpectatorCount::default());
        app.insert_resource(spectated());
        app.init_resource::<spectator::HistoryBootstrap<Vec<u8>>>();
        let bytes = bincode::serialize(&7_u8).unwrap();
        app.world_mut()
            .run_system_once_with(
                |input: In<spectator::state_machine::SpectateOutput<u8, Vec<u8>>>,
                 spectated: Option<ResMut<spectator::Spectated<MockState>>>,
                 mut count: ResMut<spectator::SpectatorCount>,
                 mut commands: Commands,
                 mut bootstrap: ResMut<spectator::HistoryBootstrap<Vec<u8>>>| {
                    let In(output) = input;
                    spectator::Machine::<u8, Vec<u8>>::carry_out_state_output(
                        &output,
                        spectated.map(ResMut::into_inner),
                        &mut count,
                        &mut commands,
                        &mut bootstrap,
                    );
                },
                spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::Merge(bytes),
            )
            .unwrap();
        let spectated = app.world().resource::<spectator::Spectated<MockState>>();
        assert_eq!(*spectated.state, Some(7));
        assert_eq!(spectated.freshness, spectator::SpectateFreshness::Live);
    }
}
