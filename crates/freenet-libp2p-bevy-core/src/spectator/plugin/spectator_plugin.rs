use std::marker::PhantomData;
use std::sync::Mutex;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;

pub struct SpectatorPlugin<S: replica_sync::traits::ReplicatedState> {
    pub game: identity::GameId,
    pub marker: PhantomData<S>,
}

impl<S: replica_sync::traits::ReplicatedState> SpectatorPlugin<S> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<S: replica_sync::traits::ReplicatedState> bevy::prelude::Plugin for SpectatorPlugin<S> {
    fn build(&self, app: &mut App) {
        let Some(libp2p_events) = app
            .world()
            .get_resource::<p2p::Libp2pEventChannel>()
            .map(p2p::Libp2pEventChannel::subscribe)
        else {
            tracing::error!(target: "spectator", "spectator: p2p Libp2pEventChannel missing");
            return;
        };
        app.add_message::<spectator::SpectateCommand>();
        app.insert_resource(spectator::SpectatorGame(self.game.clone()));
        app.insert_resource(spectator::SpectatorEvents(Mutex::new(libp2p_events)));
        app.insert_resource(spectator::SpectatorMachine(
            spectator::state_machine::SpectatorState::<S::Op, Vec<u8>>::new(self.game.clone()),
        ));
        app.insert_resource(spectator::SpectatorInputs::<S::Op, Vec<u8>>(Vec::new()));
        app.init_resource::<spectator::HistoryBootstrap<Vec<u8>>>();
        app.init_resource::<spectator::SpectatorCount>();
        app.add_systems(
            Update,
            (
                spectator::bevy_systems::sync_local_peer::<S>,
                spectator::bevy_systems::route_spectate_commands::<S>,
                spectator::bevy_systems::route_topic_messages::<S>,
                spectator::bevy_systems::count_spectators::<S>,
                spectator::bevy_systems::drive::<S>,
            )
                .chain()
                .in_set(replica_sync::ReplicaSyncSet::Drive),
        );
    }
}

#[atomic_delegates]
impl<S: replica_sync::traits::ReplicatedState> SpectatorPlugin<S> {}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::SpectatorPlugin;
    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::replica_sync;
    use crate::spectator;

    #[derive(Default, Resource)]
    struct State;

    impl replica_sync::traits::ReplicatedState for State {
        type Op = u8;
        type Snapshot = u8;

        fn apply(&mut self, _meta: replica_sync::OpMeta, _op: &Self::Op) {}

        fn snapshot(&self) -> Self::Snapshot {
            0
        }

        fn merge(&mut self, _snapshot: Self::Snapshot) {}

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            Vec::new()
        }

        fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
            0
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(plugin::Config::<u8>::new(
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        )));
        app.add_plugins(SpectatorPlugin::<State>::new(identity::GameId::from(
            "test",
        )));
        app.update();
        assert!(
            app.world()
                .get_resource::<spectator::SpectatorMachine<u8, Vec<u8>>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<spectator::SpectatorCount>()
                .is_some()
        );
    }
}
