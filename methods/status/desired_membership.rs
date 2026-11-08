use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Entity;

use crate::discovery;
use crate::identity;

#[must_use]
pub fn desired_membership(
    status: &discovery::Status,
    room_entity_by_name: &BTreeMap<identity::RoomName, Entity>,
) -> (
    BTreeMap<identity::PeerId, Entity>,
    BTreeMap<identity::PeerId, BTreeSet<Entity>>,
) {
    let mut member_views: Vec<(&discovery::Members, Entity)> = Vec::new();
    if let Some(joined) = status.room.as_ref()
        && let Some(room_entity) = room_entity_by_name.get(&joined.name).copied()
    {
        member_views.push((&joined.members, room_entity));
    }
    for (room_name, lobby_room) in status.lobby.iter() {
        if status
            .room
            .as_ref()
            .is_some_and(|joined| &joined.name == room_name)
        {
            continue;
        }
        if let Some(room_entity) = room_entity_by_name.get(room_name).copied() {
            member_views.push((&lobby_room.members, room_entity));
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
    if let Some(self_id) = status.self_peer_id.as_ref() {
        let me_wants_room = status
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
    use crate::time;

    fn occupant(id: &str) -> (identity::PeerId, discovery::RoomMember) {
        let peer_id = identity::PeerId::from(id);
        (
            peer_id.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer_id,
                    addrs: Vec::new(),
                },
                updated_at: time::UnixTime::from(10),
                display_name: None,
                membership: discovery::RoomMembership::Occupant,
            },
        )
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-a");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from(10),
                members: BTreeMap::from([occupant("alice")]),
            },
        );
        let status = discovery::Status {
            lobby,
            ..discovery::Status::default()
        };
        let by_name = BTreeMap::from([(room, Entity::PLACEHOLDER)]);
        let (in_room, holds) = desired_membership(&status, &by_name);
        assert_eq!(
            in_room,
            BTreeMap::from([(identity::PeerId::from("alice"), Entity::PLACEHOLDER)])
        );
        assert_eq!(holds, BTreeMap::new());
    }

    #[test]
    fn joined_room_wins_over_lobby_room() {
        let room_a = identity::RoomName::from("room-a");
        let room_b = identity::RoomName::from("room-b");
        let mut lobby = discovery::Lobby::default();
        for (room, members) in [
            (&room_a, vec![occupant("alice")]),
            (&room_b, vec![occupant("alice"), occupant("bob")]),
        ] {
            lobby.insert(
                room.clone(),
                discovery::LobbyRoom {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::from(10),
                    members: members.into_iter().collect(),
                },
            );
        }
        let status = discovery::Status {
            lobby,
            room: Some(discovery::JoinedRoom {
                name: room_a.clone(),
                created_at: time::UnixTime::from(10),
                phase: discovery::JoinPhase::Ready,
                members: BTreeMap::from([occupant("alice")]),
                how_entered: discovery::HowRoomWasEntered::Joined,
            }),
            self_peer_id: Some(identity::PeerId::from("me")),
            ..discovery::Status::default()
        };
        let joined = Entity::from_bits(1);
        let other = Entity::from_bits(2);
        let by_name = BTreeMap::from([(room_a, joined), (room_b, other)]);
        let (in_room, _) = desired_membership(&status, &by_name);
        assert_eq!(in_room.get(&identity::PeerId::from("alice")), Some(&joined));
        assert_eq!(in_room.get(&identity::PeerId::from("bob")), Some(&other));
    }
}
