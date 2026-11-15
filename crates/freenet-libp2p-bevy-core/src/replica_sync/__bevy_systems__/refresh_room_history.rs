use bevy::prelude::*;

use crate::replica_sync;

pub fn refresh_room_history<Replica: replica_sync::traits::ReplicatedState>(
    time: Res<Time>,
    mut timer: ResMut<replica_sync::HistoryWriteTimer>,
    mut writer: MessageWriter<replica_sync::ReplicaInputFromTimer<Replica::Op>>,
) {
    let time = time.into_inner();
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    writer.write(replica_sync::ReplicaInputFromTimer(
        replica_sync::state_machine::Input::HistoryWriteTimerFired,
    ));
}

// no test_usage necessary
