use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{Commands, MessageWriter, Resource};
use derive_more::{Deref, DerefMut};

use crate::p2p;
use crate::replica_sync;
use crate::spectator;

#[derive(Resource, Deref, DerefMut)]
pub struct Machine<Op, Snap>(pub spectator::state_machine::SpectatorState<Op, Snap>);

#[atomic_delegates]
impl<Op> Machine<Op, Vec<u8>> {
    pub(crate) fn carry_out_network_output<S: replica_sync::traits::ReplicatedState<Op = Op>>(
        &self,
        output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
        game: &spectator::SpectatorGame,
        spectated: Option<&mut spectator::Spectated<S>>,
        commands: &mut Commands,
        bootstrap: &mut spectator::HistoryBootstrap<Vec<u8>>,
        network: &mut MessageWriter<p2p::Libp2pCommand>,
    ) {
    }
    pub(crate) fn carry_out_state_output<S: replica_sync::traits::ReplicatedState<Op = Op>>(
        output: &spectator::state_machine::SpectateOutput<S::Op, Vec<u8>>,
        spectated: Option<&mut spectator::Spectated<S>>,
        count: &mut spectator::SpectatorCount,
        commands: &mut Commands,
        bootstrap: &mut spectator::HistoryBootstrap<Vec<u8>>,
    ) {
    }
}

// no test_usage necessary
