use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;

pub fn drop_members_whose_lease_lapsed(
    lobby: &mut discovery::Lobby,
    observed: &BTreeMap<
        (identity::RoomName, identity::PeerId),
        discovery::state_machine::ObservedLobbyHeartbeat,
    >,
    now: discovery::UnixTime,
    limits: discovery::LobbyLeaseLimits,
) {
    let stale = limits
        .lease_ttl
        .saturating_add(limits.remote_clock_skew_margin);
    for (room_name, record) in lobby.iter_mut() {
        record.members.retain(|peer_id, room_member| {
            if room_member.membership == discovery::RoomMembership::Departed {
                return false;
            }
            let key = (room_name.clone(), peer_id.clone());
            let first_seen = observed
                .get(&key)
                .map_or(now, |seen| seen.first_seen_locally_at);
            if now.elapsed_since(first_seen) > limits.lease_ttl {
                return false;
            }
            if now.elapsed_since(room_member.updated_at) > stale {
                return false;
            }
            true
        });
    }
    lobby.retain(|_, record| !record.members.is_empty());
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use super::drop_members_whose_lease_lapsed;
    use crate::discovery;
    use crate::identity;

    fn room_member(
        updated_at: discovery::UnixTime,
        membership: discovery::RoomMembership,
    ) -> discovery::RoomMember {
        discovery::RoomMember {
            peer: identity::Peer {
                id: identity::PeerId::from("p"),
                addrs: Vec::new(),
            },
            updated_at,
            display_name: None,
            membership,
        }
    }

    fn lobby_with(room: &str, peer: &str, member: discovery::RoomMember) -> discovery::Lobby {
        discovery::Lobby::from([(
            identity::RoomName::from(room),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from(0),
                members: BTreeMap::from([(identity::PeerId::from(peer), member)]),
            },
        )])
    }

    fn limits() -> discovery::LobbyLeaseLimits {
        discovery::LobbyLeaseLimits {
            lease_ttl: Duration::from_secs(90),
            remote_clock_skew_margin: Duration::from_secs(600),
        }
    }

    #[test]
    fn test_usage() {
        let mut lobby = lobby_with(
            "r",
            "p",
            room_member(
                discovery::UnixTime::from(190),
                discovery::RoomMembership::Occupant,
            ),
        );
        drop_members_whose_lease_lapsed(
            &mut lobby,
            &BTreeMap::new(),
            discovery::UnixTime::from(200),
            limits(),
        );
        assert_eq!(lobby.len(), 1);
    }

    #[test]
    fn holder_with_skewed_clock_dies_one_lease_after_its_last_observed_heartbeat() {
        let remote_updated_at = discovery::UnixTime::from(1000);
        let first_seen = discovery::UnixTime::from(700);
        let observed = BTreeMap::from([(
            (identity::RoomName::from("r"), identity::PeerId::from("p")),
            discovery::state_machine::ObservedLobbyHeartbeat {
                remote_updated_at,
                first_seen_locally_at: first_seen,
            },
        )]);
        let mut lobby = discovery::Lobby::from([(
            identity::RoomName::from("r"),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from(0),
                members: BTreeMap::from([(
                    identity::PeerId::from("p"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("p"),
                            addrs: Vec::new(),
                        },
                        updated_at: remote_updated_at,
                        display_name: None,
                        membership: discovery::RoomMembership::HistoryHolder,
                    },
                )]),
            },
        )]);
        drop_members_whose_lease_lapsed(
            &mut lobby,
            &observed,
            discovery::UnixTime::from(789),
            limits(),
        );
        assert_eq!(lobby.len(), 1);
        drop_members_whose_lease_lapsed(
            &mut lobby,
            &observed,
            discovery::UnixTime::from(791),
            limits(),
        );
        assert!(lobby.is_empty());
    }

    #[test]
    fn holder_five_minutes_behind_that_keeps_advancing_stays_alive() {
        let now = discovery::UnixTime::from(1000);
        let remote_updated_at = discovery::UnixTime::from(700);
        let observed = BTreeMap::from([(
            (identity::RoomName::from("r"), identity::PeerId::from("p")),
            discovery::state_machine::ObservedLobbyHeartbeat {
                remote_updated_at,
                first_seen_locally_at: now,
            },
        )]);
        let mut lobby = lobby_with(
            "r",
            "p",
            discovery::RoomMember {
                peer: identity::Peer {
                    id: identity::PeerId::from("p"),
                    addrs: Vec::new(),
                },
                updated_at: remote_updated_at,
                display_name: None,
                membership: discovery::RoomMembership::HistoryHolder,
            },
        );
        drop_members_whose_lease_lapsed(&mut lobby, &observed, now, limits());
        assert_eq!(lobby.len(), 1);
    }

    #[test]
    fn first_sighting_of_a_timestamp_older_than_lease_plus_margin_is_dropped_at_once() {
        let now = discovery::UnixTime::from(1000);
        let mut lobby = lobby_with(
            "r",
            "p",
            room_member(
                discovery::UnixTime::from(0),
                discovery::RoomMembership::HistoryHolder,
            ),
        );
        drop_members_whose_lease_lapsed(&mut lobby, &BTreeMap::new(), now, limits());
        assert!(lobby.is_empty());
    }
}
