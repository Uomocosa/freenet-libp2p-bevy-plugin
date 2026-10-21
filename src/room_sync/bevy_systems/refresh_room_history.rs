use bevy::prelude::*;

use crate::p2p;
use crate::room_sync;

pub fn refresh_room_history<Replica: room_sync::RoomState>(
    time: Res<Time>,
    mut timer: ResMut<room_sync::HistoryWriteTimer>,
    mut state: ResMut<room_sync::bevy_systems::SyncState<Replica::Op>>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let time = time.into_inner();
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    for output in room_sync::state_machine::update(
        &mut state,
        room_sync::state_machine::Input::HistoryWriteTimerFired,
    ) {
        if let room_sync::state_machine::Output::GetHistoryRecord { id } = output {
            pending.push(p2p::NetCommand::GetHistoryRecord { id });
        }
    }
}

// no test_usage necessary
