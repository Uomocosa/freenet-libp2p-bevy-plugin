use bevy::prelude::*;

use crate::discovery;
use crate::p2p;
use crate::peek;
use crate::spectator;
use crate::ui;

use ui::discovery::{SelectedRoom, StreamedRoom};

pub fn peek_selected_room(
    selection: Option<Res<SelectedRoom>>,
    streamed: Option<Res<StreamedRoom>>,
    discovery: Option<Res<discovery::state_machine::DiscoveryState>>,
    game: Option<Res<peek::RoomPeekGame>>,
    time: Res<Time>,
    mut started_room: Local<Option<discovery::RoomName>>,
    mut cleared_for_secs: Local<Option<f32>>,
    mut spectate: MessageWriter<spectator::SpectateCommand>,
    network: Option<MessageWriter<p2p::Libp2pCommand>>,
) {
    let Some(selection) = selection else {
        return;
    };
    let streamed_room = streamed
        .map(Res::into_inner)
        .and_then(|streamed| (**streamed).clone());
    let preview_room = (**selection)
        .clone()
        .filter(|room| streamed_room.as_ref() != Some(room));
    let Some(room) = preview_room else {
        if started_room.is_none() {
            return;
        }
        let elapsed = cleared_for_secs.unwrap_or(0.0) + time.into_inner().delta_secs();
        if elapsed < peek::PREVIEW_STOP_LINGER_SECS {
            *cleared_for_secs = Some(elapsed);
            return;
        }
        *started_room = None;
        *cleared_for_secs = None;
        spectate.write(spectator::SpectateCommand::Stop {
            slot: spectator::SpectateSlot::Preview,
        });
        return;
    };
    *cleared_for_secs = None;
    if started_room.as_ref() == Some(&room) {
        return;
    }
    let (Some(state), Some(game)) = (discovery.map(Res::into_inner), game.map(Res::into_inner))
    else {
        return;
    };
    let Some(room_record) = state.lobby.get(&room) else {
        return;
    };
    let history_prefix = p2p::HistoryKeyPrefix::new(game, &room, room_record.created_at);
    *started_room = Some(room.clone());
    if let Some(mut network) = network
        && let Some(member) = peek::bevy_systems::rendezvous_preview_dial::rendezvous_preview_dial(
            &state.me.id,
            room_record.members.values(),
        )
    {
        network.write(p2p::Libp2pCommand::Dial(member.peer.clone()));
    }
    spectate.write(spectator::SpectateCommand::Start {
        slot: spectator::SpectateSlot::Preview,
        zone: (*room).clone(),
        history_prefix,
    });
}
