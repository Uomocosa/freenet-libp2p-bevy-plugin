use bevy::prelude::*;

use crate::discovery;
use crate::p2p;
use crate::peek;
use crate::spectator;
use crate::ui;

use ui::discovery::StreamedRoom;

pub fn stream_streamed_room(
    streamed: Option<Res<StreamedRoom>>,
    discovery: Option<Res<discovery::state_machine::DiscoveryState>>,
    game: Option<Res<peek::RoomPeekGame>>,
    mut started_room: Local<Option<discovery::RoomName>>,
    mut spectate: MessageWriter<spectator::SpectateCommand>,
    network: Option<MessageWriter<p2p::Libp2pCommand>>,
) {
    let Some(streamed) = streamed else {
        return;
    };
    let Some(room) = (**streamed).clone() else {
        if started_room.is_none() {
            return;
        }
        *started_room = None;
        spectate.write(spectator::SpectateCommand::Stop {
            slot: spectator::SpectateSlot::Stream,
        });
        return;
    };
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
        slot: spectator::SpectateSlot::Stream,
        zone: (*room).clone(),
        history_prefix,
    });
}
