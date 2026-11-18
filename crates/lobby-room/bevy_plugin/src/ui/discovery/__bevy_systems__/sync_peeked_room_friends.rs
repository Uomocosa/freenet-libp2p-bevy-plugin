use crate::direct_dial;
use crate::discovery as core_discovery;
use crate::identity;
use crate::peek;
use crate::ui;
use bevy::prelude::*;

pub fn sync_peeked_room_friends(
    hovered_pill: Res<ui::discovery::HoveredOccupancyPill>,
    preview: Option<Res<peek::RoomPreviewTarget>>,
    state: Option<Res<core_discovery::state_machine::DiscoveryState>>,
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    mut peeked: ResMut<ui::discovery::PeekedRoomFriends>,
) {
    let hovered_pill = hovered_pill.into_inner();
    let friends = match (state, dial, preview) {
        (Some(state), Some(dial), Some(preview)) => (**hovered_pill)
            .clone()
            .filter(|room| preview.zone == **room)
            .map_or_else(Vec::new, |room| {
                let ids: Vec<identity::PeerId> =
                    state.lobby.get(&room).map_or_else(Vec::new, |record| {
                        record
                            .members
                            .keys()
                            .filter(|id| dial.is_friend(id))
                            .cloned()
                            .collect()
                    });
                state.room_friends(&room, &ids)
            }),
        _ => Vec::new(),
    };
    peeked.set_if_neq(ui::discovery::PeekedRoomFriends(friends));
}
