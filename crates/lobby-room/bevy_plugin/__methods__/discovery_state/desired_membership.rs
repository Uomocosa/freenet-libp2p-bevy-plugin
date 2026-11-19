use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Entity;

use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;

#[must_use]
pub fn desired_membership(
    state: &DiscoveryState,
    room_entity_by_name: &BTreeMap<discovery::RoomName, Entity>,
) -> (
    BTreeMap<identity::PeerId, Entity>,
    BTreeMap<identity::PeerId, BTreeSet<Entity>>,
) {
    let mut member_views: Vec<(&discovery::Members, Entity)> = Vec::new();
    if let Some(joined) = state.room.as_ref()
        && let Some(room_entity) = room_entity_by_name.get(&joined.name).copied()
    {
        member_views.push((&joined.members, room_entity));
    }
    for (room_name, room) in state.lobby.iter() {
        if state
            .room
            .as_ref()
            .is_some_and(|joined| &joined.name == room_name)
        {
            continue;
        }
        if let Some(room_entity) = room_entity_by_name.get(room_name).copied() {
            member_views.push((&room.members, room_entity));
        }
    }
    let mut desired_in_room: BTreeMap<identity::PeerId, Entity> = BTreeMap::new();
    let mut desired_holds: BTreeMap<identity::PeerId, BTreeSet<Entity>> = BTreeMap::new();
    for (members, room_entity) in member_views {
        for (peer, member) in members {
            match member.membership {
                discovery::RoomMembership::Occupant => {
                    desired_in_room.entry(peer.clone()).or_insert(room_entity);
                }
                discovery::RoomMembership::HistoryHolder => {
                    desired_holds
                        .entry(peer.clone())
                        .or_default()
                        .insert(room_entity);
                }
                discovery::RoomMembership::Departed => {}
            }
        }
    }
    let self_id = &state.me.id;
    let me_wants_room = state
        .room
        .as_ref()
        .and_then(|joined| room_entity_by_name.get(&joined.name).copied());
    match me_wants_room {
        Some(room_entity) => {
            desired_in_room.insert(self_id.clone(), room_entity);
        }
        None => {
            desired_in_room.remove(self_id);
        }
    }
    (desired_in_room, desired_holds)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bevy::prelude::Entity;

    use super::desired_membership;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let room = discovery::RoomName::from("room-a");
        let lobby = testing::lobby_with(vec![(
            "room-a",
            testing::room(
                discovery::RoomSize::DEFAULT,
                10,
                testing::room_members(&[("alice", None, discovery::RoomMembership::Occupant)]),
            ),
        )]);
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        let by_name = BTreeMap::from([(room, Entity::PLACEHOLDER)]);
        let (in_room, holds) = desired_membership(&state, &by_name);
        assert_eq!(
            in_room,
            BTreeMap::from([(identity::PeerId::from("alice"), Entity::PLACEHOLDER)])
        );
        assert_eq!(holds, BTreeMap::new());
    }

    #[test]
    fn joined_room_wins_over_lobby_room() {
        let room_a = discovery::RoomName::from("room-a");
        let room_b = discovery::RoomName::from("room-b");
        let lobby = testing::lobby_with(vec![
            (
                "room-a",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    10,
                    testing::room_members(&[("alice", None, discovery::RoomMembership::Occupant)]),
                ),
            ),
            (
                "room-b",
                testing::room(
                    discovery::RoomSize::DEFAULT,
                    10,
                    testing::room_members(&[
                        ("alice", None, discovery::RoomMembership::Occupant),
                        ("bob", None, discovery::RoomMembership::Occupant),
                    ]),
                ),
            ),
        ]);
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .in_room(testing::joined_room(
                "room-a",
                10,
                discovery::JoinPhase::Ready,
                testing::room_members(&[("alice", None, discovery::RoomMembership::Occupant)]),
            ))
            .build();
        let joined = Entity::from_bits(1);
        let other = Entity::from_bits(2);
        let by_name = BTreeMap::from([(room_a, joined), (room_b, other)]);
        let (in_room, _) = desired_membership(&state, &by_name);
        assert_eq!(in_room.get(&identity::PeerId::from("alice")), Some(&joined));
        assert_eq!(in_room.get(&identity::PeerId::from("bob")), Some(&other));
    }
}
