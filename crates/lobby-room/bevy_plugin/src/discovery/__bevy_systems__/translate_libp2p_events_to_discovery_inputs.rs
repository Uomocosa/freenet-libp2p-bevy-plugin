use bevy::prelude::*;

use crate::discovery;

pub fn translate_libp2p_events_to_discovery_inputs(
    mut tap: Option<ResMut<discovery::Libp2pEventTap>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    let Some(tap) = tap.as_deref_mut() else {
        return;
    };
    while let Ok(event) = tap.try_recv() {
        inputs.write(discovery::state_machine::Input::Libp2pEvent(event));
    }
}
