use bevy::prelude::*;

use crate::replica_sync;

pub fn publish_slice_content_hashes<Replica: replica_sync::traits::ReplicatedState>(
    time: Res<Time>,
    mut timer: ResMut<replica_sync::AntiEntropyTimer>,
    replica: Res<Replica>,
    mut writer: MessageWriter<replica_sync::ReplicaInputFromTimer<Replica::Op>>,
) {
    let time = time.into_inner();
    let replica = replica.into_inner();
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    writer.write(replica_sync::ReplicaInputFromTimer(
        replica_sync::state_machine::Input::AntiEntropyTimerFired {
            local: replica.slice_content_hashes(),
        },
    ));
}

// no test_usage necessary
