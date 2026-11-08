use bevy::prelude::Entity;
use bevy::prelude::{Has, Query, With};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::room_sync;
use crate::time;

pub fn room_inputs_from_entities<Op>(
    state: &mut room_sync::bevy_systems::SyncState<Op>,
    game: &identity::GameId,
    me_rooms: &Query<&peers::InRoom, With<peers::Me>>,
    room_details: &Query<room_sync::RoomDetails<'_>>,
    member_peers: &Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
) -> (
    Vec<p2p::Libp2pCommand>,
    Vec<room_sync::state_machine::Input<Op>>,
) {
    let current_room_entity = me_rooms.iter().next().map(|in_room| **in_room);
    let current = current_room_entity.and_then(|entity| {
        let (room_name, prefix, liveness, members) = room_details.get(entity).ok()?;
        let room_name = (**room_name).clone();
        let prefix = prefix.cloned().unwrap_or_else(|| {
            p2p::HistoryKeyPrefix::new(game, &room_name, time::UnixTime::from(0))
        });
        let liveness = liveness
            .copied()
            .unwrap_or(room_sync::state_machine::RoomLiveness {
                is_room_still_listed: true,
                has_other_history_holders: false,
            });
        let mut member_ids: Vec<identity::PeerId> = members.map_or_else(Vec::new, |members| {
            let members: &[Entity] = members;
            members
                .iter()
                .filter_map(|member| member_peers.get(*member).ok())
                .filter(|(_, is_connected, is_me)| *is_connected && !*is_me)
                .map(|(peer, _, _)| (**peer).clone())
                .collect()
        });
        member_ids.sort();
        Some((room_name, prefix, member_ids, liveness))
    });
    let mut commands: Vec<p2p::Libp2pCommand> = Vec::new();
    let mut inputs: Vec<room_sync::state_machine::Input<Op>> = Vec::new();
    match (state.room.clone(), current) {
        (None, None) => {}
        (Some(previous), None) => {
            commands.push(p2p::Libp2pCommand::Unsubscribe {
                topic: identity::Topic::from_room_sync(game, &previous),
            });
            state.reset_after_leaving();
        }
        (None, Some((room_name, prefix, members, liveness))) => {
            commands.push(p2p::Libp2pCommand::Subscribe {
                topic: identity::Topic::from_room_sync(game, &room_name),
            });
            inputs.push(room_sync::state_machine::Input::Joined {
                room: room_name,
                prefix,
                members,
                liveness,
            });
        }
        (Some(previous), Some((room_name, prefix, members, liveness))) if previous != room_name => {
            commands.push(p2p::Libp2pCommand::Unsubscribe {
                topic: identity::Topic::from_room_sync(game, &previous),
            });
            commands.push(p2p::Libp2pCommand::Subscribe {
                topic: identity::Topic::from_room_sync(game, &room_name),
            });
            inputs.push(room_sync::state_machine::Input::Joined {
                room: room_name,
                prefix,
                members,
                liveness,
            });
        }
        (Some(_), Some((_, prefix, members, liveness))) => {
            let members_changed = state.member_snapshot.members != members;
            let prefix_changed = state.kademlia_history.prefix.as_ref() != Some(&prefix);
            let liveness_changed = state.kademlia_history.liveness != liveness;
            if members_changed || prefix_changed || liveness_changed {
                inputs.push(room_sync::state_machine::Input::Members {
                    members,
                    prefix,
                    liveness,
                });
            }
        }
    }
    (commands, inputs)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::room_sync;
    use crate::time;

    #[derive(Resource, Default)]
    struct Captured {
        commands: Vec<p2p::Libp2pCommand>,
        inputs: Vec<room_sync::state_machine::Input<u8>>,
    }

    fn capture(
        mut state: ResMut<room_sync::bevy_systems::SyncState<u8>>,
        game: Res<room_sync::SyncGame>,
        me_rooms: Query<&peers::InRoom, With<peers::Me>>,
        room_details: Query<room_sync::RoomDetails<'_>>,
        member_peers: Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
        mut captured: ResMut<Captured>,
    ) {
        let game = game.into_inner();
        let (commands, inputs) =
            room_sync::bevy_systems::SyncState::<u8>::room_inputs_from_entities(
                &mut state,
                game,
                &me_rooms,
                &room_details,
                &member_peers,
            );
        captured.commands = commands;
        captured.inputs = inputs;
    }

    fn app_with_capture(game: identity::GameId) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(room_sync::SyncGame(game));
        app.init_resource::<room_sync::bevy_systems::SyncState<u8>>();
        app.init_resource::<Captured>();
        app.add_systems(Update, capture);
        app
    }

    fn join(
        app: &mut App,
        room: &identity::RoomName,
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        liveness: room_sync::state_machine::RoomLiveness,
    ) -> Entity {
        let room_entity = app
            .world_mut()
            .spawn((peers::Room::from(room.clone()), prefix, liveness))
            .id();
        app.world_mut().spawn((
            peers::Peer::from(identity::PeerId::from("me")),
            peers::Me,
            peers::InRoom(room_entity),
        ));
        for member in members {
            app.world_mut().spawn((
                peers::Peer::from(member),
                peers::InRoom(room_entity),
                peers::Connected,
            ));
        }
        room_entity
    }

    fn captured(
        app: &App,
    ) -> (
        Vec<p2p::Libp2pCommand>,
        Vec<room_sync::state_machine::Input<u8>>,
    ) {
        let captured = app.world().resource::<Captured>();
        (captured.commands.clone(), captured.inputs.clone())
    }

    #[test]
    fn test_usage() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room");
        let prefix = p2p::HistoryKeyPrefix::new(&game, &room, time::UnixTime::from(7));
        let liveness = room_sync::state_machine::RoomLiveness {
            is_room_still_listed: true,
            has_other_history_holders: true,
        };
        let mut app = app_with_capture(game);
        app.update();
        assert_eq!(captured(&app), (Vec::new(), Vec::new()));
        let room_entity = join(
            &mut app,
            &room,
            vec![identity::PeerId::from("alice")],
            prefix,
            liveness,
        );
        app.update();
        let (commands, inputs) = captured(&app);
        assert_eq!(commands.len(), 1);
        assert!(matches!(
            inputs.as_slice(),
            [room_sync::state_machine::Input::Joined { .. }]
        ));
        let joined = inputs.into_iter().next().expect("joined");
        app.world_mut()
            .resource_mut::<room_sync::bevy_systems::SyncState<u8>>()
            .update(joined, room_sync::ElapsedSinceAppStart::default());
        app.update();
        assert_eq!(captured(&app), (Vec::new(), Vec::new()));
        app.world_mut()
            .entity_mut(room_entity)
            .insert(room_sync::state_machine::RoomLiveness {
                is_room_still_listed: false,
                has_other_history_holders: false,
            });
        app.update();
        let (_, inputs) = captured(&app);
        assert!(matches!(
            inputs.as_slice(),
            [room_sync::state_machine::Input::Members { .. }]
        ));
        let me_entity = app
            .world_mut()
            .query_filtered::<Entity, With<peers::Me>>()
            .iter(app.world())
            .next()
            .expect("me");
        app.world_mut()
            .entity_mut(me_entity)
            .remove::<peers::InRoom>();
        app.update();
        let (commands, inputs) = captured(&app);
        assert_eq!(commands.len(), 1);
        assert_eq!(inputs, Vec::new());
        assert_eq!(
            app.world()
                .resource::<room_sync::bevy_systems::SyncState<u8>>()
                .room,
            None
        );
    }
}
