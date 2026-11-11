use crate::discovery;
use crate::identity;
use discovery::state_machine::DiscoveryState;
use std::collections::BTreeSet;

pub fn room_roster(state: &DiscoveryState, is_swarm_ready: bool) -> discovery::RoomRoster {
    let Some(room) = state.room.as_ref() else {
        return discovery::RoomRoster::default();
    };
    if !is_swarm_ready {
        return discovery::RoomRoster::default();
    }
    let own = match room.phase {
        discovery::JoinPhase::Joining { .. } => discovery::OwnRoomStatus::Connecting,
        discovery::JoinPhase::Ready => discovery::OwnRoomStatus::Connected,
    };
    let lobby_record = state.lobby.get(&room.name);
    let mut present_ids: BTreeSet<identity::PeerId> = state.occupants().into_iter().collect();
    if let Some(record) = lobby_record {
        for (peer, member) in &record.members {
            if member.membership == discovery::RoomMembership::Occupant {
                present_ids.insert(peer.clone());
            }
        }
    }
    let entries: Vec<discovery::RoomRosterEntry> = present_ids
        .iter()
        .map(|id| {
            let is_me = *id == state.me.id;
            let display_name = if is_me {
                Some(&state.display_name)
            } else {
                room.members
                    .get(id)
                    .and_then(|member| member.display_name.as_ref())
                    .or_else(|| {
                        lobby_record
                            .and_then(|record| record.members.get(id))
                            .and_then(|member| member.display_name.as_ref())
                    })
            };
            let name = display_name
                .filter(|username| !username.is_empty())
                .map_or_else(
                    || identity::ShortPeerId::from(id).to_string(),
                    ToString::to_string,
                );
            let status = if is_me {
                match own {
                    discovery::OwnRoomStatus::Connecting => discovery::MemberRoomStatus::Connecting,
                    discovery::OwnRoomStatus::Connected
                    | discovery::OwnRoomStatus::Disconnected => {
                        discovery::MemberRoomStatus::Connected
                    }
                }
            } else if state.connected.contains(id) {
                discovery::MemberRoomStatus::Connected
            } else {
                discovery::MemberRoomStatus::Connecting
            };
            discovery::RoomRosterEntry {
                id: id.clone(),
                name,
                status,
                is_me,
            }
        })
        .collect();
    let mut roster = discovery::RoomRoster {
        own,
        count: entries.len(),
        max: state
            .lobby
            .get(&room.name)
            .map_or(entries.len(), |record| usize::from(record.room_size.get())),
        entries,
    };
    roster
        .entries
        .sort_by(|a, b| a.status.cmp(&b.status).then_with(|| a.id.cmp(&b.id)));
    roster
}

#[cfg(test)]
mod tests {
    use super::room_roster;
    use crate::discovery;
    use crate::identity;
    use crate::time;
    use discovery::state_machine::DiscoveryState;
    use std::collections::BTreeMap;

    fn room_member(
        peer: &str,
        display_name: Option<&str>,
        membership: discovery::RoomMembership,
    ) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from(peer),
                addrs: Vec::new(),
            },
            display_name: display_name.map(identity::Username::from),
            updated_at: time::UnixTime::EPOCH,
            membership,
        }
    }

    fn room(
        phase: discovery::JoinPhase,
        members: Vec<(&str, discovery::RoomMember)>,
    ) -> discovery::JoinedRoom {
        discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: time::UnixTime::EPOCH,
            phase,
            members: members
                .into_iter()
                .map(|(peer, member)| (identity::PeerId::from(peer), member))
                .collect::<BTreeMap<_, _>>(),
            how_entered: discovery::HowRoomWasEntered::Created,
        }
    }

    fn state_with(
        me: &str,
        display_name: Option<&str>,
        joined: discovery::JoinedRoom,
        connected: &[&str],
    ) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from(me),
            addrs: Vec::new(),
        });
        state.display_name = identity::Username::from(display_name.unwrap_or(""));
        state.room = Some(joined);
        for peer in connected {
            state.connected.insert(identity::PeerId::from(*peer));
        }
        state
    }

    fn ready() -> discovery::JoinedRoom {
        room(
            discovery::JoinPhase::Ready,
            vec![
                (
                    "other",
                    room_member("other", Some("Other"), discovery::RoomMembership::Occupant),
                ),
                (
                    "holder",
                    room_member("holder", None, discovery::RoomMembership::HistoryHolder),
                ),
                (
                    "departed",
                    room_member("departed", None, discovery::RoomMembership::Departed),
                ),
            ],
        )
    }

    #[test]
    fn test_usage() {
        let state = state_with("me", Some("Me"), ready(), &["other"]);
        let roster = room_roster(&state, true);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Connected);
        assert_eq!(roster.count, 2);
    }

    #[test]
    fn without_a_room_the_roster_is_disconnected_and_empty() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let roster = room_roster(&state, true);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Disconnected);
        assert_eq!(roster.entries, Vec::new());
        assert_eq!(roster.count, 0);
    }

    #[test]
    fn without_a_ready_swarm_the_roster_is_disconnected_and_empty() {
        let state = state_with("me", None, ready(), &[]);
        let roster = room_roster(&state, false);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Disconnected);
        assert_eq!(roster.entries, Vec::new());
    }

    #[test]
    fn joining_own_status_connects_me_as_connecting() {
        let state = state_with(
            "me",
            None,
            room(
                discovery::JoinPhase::Joining {
                    since: time::UnixTime::EPOCH,
                },
                Vec::new(),
            ),
            &[],
        );
        let roster = room_roster(&state, true);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Connecting);
        assert_eq!(roster.entries.len(), 1);
        assert!(roster.entries[0].is_me);
        assert_eq!(
            roster.entries[0].status,
            discovery::MemberRoomStatus::Connecting
        );
    }

    #[test]
    fn entries_hold_occupants_only_connecting_first() {
        let state = state_with("me", Some("Me"), ready(), &[]);
        let roster = room_roster(&state, true);
        let statuses: Vec<(discovery::MemberRoomStatus, bool)> = roster
            .entries
            .iter()
            .map(|entry| (entry.status, entry.is_me))
            .collect();
        assert_eq!(
            statuses,
            vec![
                (discovery::MemberRoomStatus::Connecting, false),
                (discovery::MemberRoomStatus::Connected, true),
            ]
        );
        let names: Vec<&str> = roster
            .entries
            .iter()
            .map(|entry| entry.name.as_str())
            .collect();
        assert_eq!(names, vec!["Other", "Me"]);
    }

    #[test]
    fn unconnected_occupants_show_their_display_name() {
        let mut joined = ready();
        joined.members.insert(
            identity::PeerId::from("anon"),
            room_member("anon", None, discovery::RoomMembership::Occupant),
        );
        let state = state_with("me", Some("Me"), joined, &[]);
        let roster = room_roster(&state, true);
        let anon = roster
            .entries
            .iter()
            .find(|entry| entry.id == identity::PeerId::from("anon"))
            .expect("anon");
        assert_eq!(anon.name, "anon");
        assert_eq!(anon.status, discovery::MemberRoomStatus::Connecting);
        assert_eq!(roster.count, 3);
    }

    #[test]
    fn lobby_record_occupants_not_yet_connected_show_connecting() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::try_from(8).expect("8"),
                created_at: time::UnixTime::EPOCH,
                members: [
                    (
                        "far",
                        room_member("far", Some("Far"), discovery::RoomMembership::Occupant),
                    ),
                    (
                        "holder",
                        room_member("holder", None, discovery::RoomMembership::HistoryHolder),
                    ),
                    (
                        "gone",
                        room_member("gone", None, discovery::RoomMembership::Departed),
                    ),
                ]
                .into_iter()
                .map(|(peer, member)| (identity::PeerId::from(peer), member))
                .collect::<BTreeMap<_, _>>(),
            },
        );
        let mut state = state_with("me", Some("Me"), ready(), &["other"]);
        state.lobby = lobby;
        let roster = room_roster(&state, true);
        let ids: Vec<identity::PeerId> = roster
            .entries
            .iter()
            .map(|entry| entry.id.clone())
            .collect();
        assert_eq!(
            ids,
            vec![
                identity::PeerId::from("far"),
                identity::PeerId::from("me"),
                identity::PeerId::from("other"),
            ]
        );
        let far = roster
            .entries
            .iter()
            .find(|entry| entry.id == identity::PeerId::from("far"))
            .expect("far");
        assert_eq!(far.name, "Far");
        assert_eq!(far.status, discovery::MemberRoomStatus::Connecting);
        let other = roster
            .entries
            .iter()
            .find(|entry| entry.id == identity::PeerId::from("other"))
            .expect("other");
        assert_eq!(other.status, discovery::MemberRoomStatus::Connected);
        assert_eq!(other.name, "Other");
        let me = roster
            .entries
            .iter()
            .find(|entry| entry.id == identity::PeerId::from("me"))
            .expect("me");
        assert_eq!(me.status, discovery::MemberRoomStatus::Connected);
        assert_eq!(roster.count, 3);
    }

    #[test]
    fn lobby_record_occupants_do_not_duplicate_connected_members() {
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::try_from(8).expect("8"),
                created_at: time::UnixTime::EPOCH,
                members: std::iter::once((
                    "other",
                    room_member("other", Some("Other"), discovery::RoomMembership::Occupant),
                ))
                .map(|(peer, member)| (identity::PeerId::from(peer), member))
                .collect::<BTreeMap<_, _>>(),
            },
        );
        let mut state = state_with("me", Some("Me"), ready(), &["other"]);
        state.lobby = lobby;
        let roster = room_roster(&state, true);
        let ids: Vec<identity::PeerId> = roster
            .entries
            .iter()
            .map(|entry| entry.id.clone())
            .collect();
        assert_eq!(
            ids,
            vec![
                identity::PeerId::from("me"),
                identity::PeerId::from("other"),
            ]
        );
        assert_eq!(roster.count, 2);
    }

    #[test]
    fn max_comes_from_the_lobby_record() {
        let mut state = state_with("me", None, ready(), &["other"]);
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            identity::RoomName::from("alpha"),
            discovery::Room {
                room_size: discovery::RoomSize::try_from(8).expect("8"),
                created_at: time::UnixTime::EPOCH,
                members: BTreeMap::new(),
            },
        );
        state.lobby = lobby;
        let roster = room_roster(&state, true);
        assert_eq!(roster.max, 8);
        assert_eq!(roster.count, 2);
    }
}
