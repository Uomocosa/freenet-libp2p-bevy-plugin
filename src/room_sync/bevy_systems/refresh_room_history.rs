use bevy::prelude::*;

use crate::room_sync;

pub fn refresh_room_history<Replica: room_sync::RoomState>(
    time: Res<Time>,
    mut timer: ResMut<room_sync::HistoryWriteTimer>,
    mut writer: MessageWriter<room_sync::ReplicaInputFromTimer<Replica::Op>>,
) {
    let time = time.into_inner();
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    writer.write(room_sync::ReplicaInputFromTimer(
        room_sync::state_machine::Input::HistoryWriteTimerFired,
    ));
}

// no test_usage necessary
