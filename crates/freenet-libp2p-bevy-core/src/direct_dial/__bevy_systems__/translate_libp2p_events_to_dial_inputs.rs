use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::p2p;

pub fn translate_libp2p_events_to_dial_inputs(
    mut tap: ResMut<direct_dial::DialEventTap>,
    mut inputs: MessageWriter<direct_dial::state_machine::Input>,
) {
    while let Ok(event) = tap.try_recv() {
        match event {
            p2p::Libp2pEvent::PeerConnected(id) => {
                inputs.write(direct_dial::state_machine::Input::PeerConnected(id));
            }
            p2p::Libp2pEvent::PeerDisconnected(id) => {
                inputs.write(direct_dial::state_machine::Input::PeerDisconnected(id));
            }
            p2p::Libp2pEvent::DialFailed { peer_id, .. } => {
                inputs.write(direct_dial::state_machine::Input::DialFailed(peer_id));
            }
            p2p::Libp2pEvent::PeerListenAddrs { peer_id, addrs } => {
                inputs.write(direct_dial::state_machine::Input::PeerListenAddrs(
                    identity::Peer { id: peer_id, addrs },
                ));
            }
            _ => {}
        }
    }
}
