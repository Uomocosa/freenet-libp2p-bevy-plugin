use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn publish_slice_content_hashes<Replica: room_sync::RoomState>(
    time: Res<Time>,
    mut timer: ResMut<room_sync::AntiEntropyTimer>,
    replica: Res<Replica>,
    game: Res<room_sync::SyncGame>,
    mut state: ResMut<room_sync::bevy_systems::SyncState<Replica::Op>>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let time = time.into_inner();
    let game = game.into_inner();
    let replica = replica.into_inner();
    if !timer.tick(time.delta()).just_finished() {
        return;
    }
    let input = room_sync::state_machine::Input::AntiEntropyTimerFired {
        local: replica.slice_content_hashes(),
    };
    for output in state.update(input) {
        let room_sync::state_machine::Output::PublishSliceContentHashes { room, hashes } = output
        else {
            continue;
        };
        let message =
            room_sync::ReplicaSyncTopicMessage::<Replica::Op>::CompareSliceContentHashes {
                room: room.clone(),
                hashes,
            };
        if let Ok(data) = bincode::serialize(&message) {
            pending.push(p2p::NetCommand::Publish {
                topic: identity::Topic::from_room_sync(game, &room),
                data,
            });
        }
    }
}

// no test_usage necessary
