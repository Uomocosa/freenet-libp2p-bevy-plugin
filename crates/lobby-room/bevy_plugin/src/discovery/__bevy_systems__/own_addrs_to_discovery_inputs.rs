use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;

pub fn own_addrs_to_discovery_inputs(
    ready: Option<ResMut<p2p::ReadyChannel>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    let Some(mut ready) = ready else {
        return;
    };
    let Ok(has_changed) = ready.observed_rx.has_changed() else {
        return;
    };
    if !has_changed {
        return;
    }
    let Some(addrs) = ready.observed_rx.borrow_and_update().clone() else {
        return;
    };
    inputs.write(discovery::state_machine::Input::OwnAddrsChanged(
        identity::PeerAddr::dialable(addrs),
    ));
}
