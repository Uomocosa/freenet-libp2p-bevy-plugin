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
    use crate::testing;

    #[test]
    fn test_usage() {
        let state = testing::ready_state(
            "me",
            "alpha",
            Some("Me"),
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &["other"],
        );
        let roster = room_roster(&state, true);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Connected);
        assert_eq!(roster.count, 2);
    }

    #[test]
    fn without_a_room_the_roster_is_disconnected_and_empty() {
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build();
        let roster = room_roster(&state, true);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Disconnected);
        assert_eq!(roster.entries, Vec::new());
        assert_eq!(roster.count, 0);
    }

    #[test]
    fn without_a_ready_swarm_the_roster_is_disconnected_and_empty() {
        let state = testing::ready_state(
            "me",
            "alpha",
            None,
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &[],
        );
        let roster = room_roster(&state, false);
        assert_eq!(roster.own, discovery::OwnRoomStatus::Disconnected);
        assert_eq!(roster.entries, Vec::new());
    }

    #[test]
    fn joining_own_status_connects_me_as_connecting() {
        let state = testing::joining_state("me", "alpha", 0);
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
        let state = testing::ready_state(
            "me",
            "alpha",
            Some("Me"),
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &[],
        );
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
        let mut state = testing::ready_state(
            "me",
            "alpha",
            Some("Me"),
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &[],
        );
        if let Some(room) = state.room.as_mut() {
            room.members.insert(
                identity::PeerId::from("anon"),
                testing::room_member("anon", None, discovery::RoomMembership::Occupant, &[]),
            );
        }
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
        let mut state = testing::ready_state(
            "me",
            "alpha",
            Some("Me"),
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &["other"],
        );
        state.lobby = testing::lobby_with(vec![(
            "alpha",
            testing::room(
                discovery::RoomSize::DEFAULT,
                0,
                testing::room_members(&[
                    ("far", Some("Far"), discovery::RoomMembership::Occupant),
                    ("holder", None, discovery::RoomMembership::HistoryHolder),
                    ("gone", None, discovery::RoomMembership::Departed),
                ]),
            ),
        )]);
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
        let mut state = testing::ready_state(
            "me",
            "alpha",
            Some("Me"),
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &["other"],
        );
        state.lobby = testing::lobby_with(vec![(
            "alpha",
            testing::room(
                discovery::RoomSize::DEFAULT,
                0,
                testing::room_members(&[(
                    "other",
                    Some("Other"),
                    discovery::RoomMembership::Occupant,
                )]),
            ),
        )]);
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
        let mut state = testing::ready_state(
            "me",
            "alpha",
            None,
            testing::room_members(&[
                ("other", Some("Other"), discovery::RoomMembership::Occupant),
                ("holder", None, discovery::RoomMembership::HistoryHolder),
                ("departed", None, discovery::RoomMembership::Departed),
            ]),
            &["other"],
        );
        state.lobby = testing::lobby_with(vec![(
            "alpha",
            testing::room(discovery::RoomSize::DEFAULT, 0, Vec::new()),
        )]);
        let roster = room_roster(&state, true);
        assert_eq!(roster.max, 8);
        assert_eq!(roster.count, 2);
    }
}
