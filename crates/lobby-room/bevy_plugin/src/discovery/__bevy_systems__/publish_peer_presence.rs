use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::presence;

pub fn publish_peer_presence(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    mut presence: ResMut<presence::PeerPresence>,
) {
    if state.as_ref().is_some_and(|state| !state.is_changed()) {
        return;
    }
    let mut next = presence::PeerPresence::default();
    if let Some(state) = state {
        for (room_name, room) in state.lobby.iter() {
            for (peer_id, member) in &room.members {
                if member.membership != discovery::RoomMembership::Occupant {
                    continue;
                }
                next.entry(peer_id.clone())
                    .or_insert_with(|| presence::Whereabouts {
                        zone: identity::Zone::Restricted(identity::ZoneName::clone(room_name)),
                        display_name: member.display_name.clone(),
                    });
            }
        }
    }
    presence.set_if_neq(next);
}
