use bevy::prelude::*;

use crate::room_sync;

pub fn publish_slice_content_hashes<Replica: room_sync::RoomState>(
    time: Res<Time>,
    mut timer: ResMut<room_sync::AntiEntropyTimer>,
    replica: Res<Replica>,
    mut writer: MessageWriter<room_sync::ReplicaInputFromTimer<Replica::Op>>,
) {
    let time = time.into_inner();
    let replica = replica.into_inner();
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    writer.write(room_sync::ReplicaInputFromTimer(
        room_sync::state_machine::Input::AntiEntropyTimerFired {
            local: replica.slice_content_hashes(),
        },
    ));
}

// no test_usage necessary
