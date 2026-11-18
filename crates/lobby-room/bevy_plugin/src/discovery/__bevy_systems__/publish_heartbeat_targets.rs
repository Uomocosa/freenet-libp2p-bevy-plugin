use bevy::prelude::*;

use crate::discovery;

pub fn publish_heartbeat_targets(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    sender: Option<Res<discovery::HeartbeatTargetSender>>,
) {
    let Some(state) = state else {
        return;
    };
    let Some(sender) = sender else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    let target = state.heartbeat_targets();
    sender.send_if_modified(|current| {
        let changed = *current != target;
        if changed {
            current.clone_from(&target);
        }
        changed
    });
}
