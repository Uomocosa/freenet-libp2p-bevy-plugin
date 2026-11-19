use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::time;

pub fn drop_members_whose_lease_lapsed(
    limits: &discovery::LobbyLeaseLimits,
    lobby: &mut discovery::Lobby,
    observed: &BTreeMap<
        (discovery::RoomName, identity::PeerId),
        discovery::state_machine::ObservedLobbyHeartbeat,
    >,
    now: time::UnixTime,
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
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let member = discovery::RoomMember {
            updated_at: time::UnixTime::from_secs(190),
            ..testing::room_member("p", None, discovery::RoomMembership::Occupant, &[])
        };
        let mut lobby = testing::lobby_with(vec![(
            "r",
            testing::room(discovery::RoomSize::DEFAULT, 0, vec![member]),
        )]);
        drop_members_whose_lease_lapsed(
            &testing::lease_limits(Duration::from_secs(90), Duration::from_secs(600)),
            &mut lobby,
            &BTreeMap::new(),
            time::UnixTime::from_secs(200),
        );
        assert_eq!(lobby.len(), 1);
    }

    #[test]
    fn holder_with_skewed_clock_dies_one_lease_after_its_last_observed_heartbeat() {
        let remote_updated_at = time::UnixTime::from_secs(1000);
        let first_seen = time::UnixTime::from_secs(700);
        let observed = BTreeMap::from([(
            (discovery::RoomName::from("r"), identity::PeerId::from("p")),
            discovery::state_machine::ObservedLobbyHeartbeat {
                remote_updated_at,
                first_seen_locally_at: first_seen,
            },
        )]);
        let member = discovery::RoomMember {
            updated_at: remote_updated_at,
            ..testing::room_member("p", None, discovery::RoomMembership::HistoryHolder, &[])
        };
        let mut lobby = testing::lobby_with(vec![(
            "r",
            testing::room(discovery::RoomSize::DEFAULT, 0, vec![member]),
        )]);
        drop_members_whose_lease_lapsed(
            &testing::lease_limits(Duration::from_secs(90), Duration::from_secs(600)),
            &mut lobby,
            &observed,
            time::UnixTime::from_secs(789),
        );
        assert_eq!(lobby.len(), 1);
        drop_members_whose_lease_lapsed(
            &testing::lease_limits(Duration::from_secs(90), Duration::from_secs(600)),
            &mut lobby,
            &observed,
            time::UnixTime::from_secs(791),
        );
        assert!(lobby.is_empty());
    }

    #[test]
    fn holder_five_minutes_behind_that_keeps_advancing_stays_alive() {
        let now = time::UnixTime::from_secs(1000);
        let remote_updated_at = time::UnixTime::from_secs(700);
        let observed = BTreeMap::from([(
            (discovery::RoomName::from("r"), identity::PeerId::from("p")),
            discovery::state_machine::ObservedLobbyHeartbeat {
                remote_updated_at,
                first_seen_locally_at: now,
            },
        )]);
        let member = discovery::RoomMember {
            updated_at: remote_updated_at,
            ..testing::room_member("p", None, discovery::RoomMembership::HistoryHolder, &[])
        };
        let mut lobby = testing::lobby_with(vec![(
            "r",
            testing::room(discovery::RoomSize::DEFAULT, 0, vec![member]),
        )]);
        drop_members_whose_lease_lapsed(
            &testing::lease_limits(Duration::from_secs(90), Duration::from_secs(600)),
            &mut lobby,
            &observed,
            now,
        );
        assert_eq!(lobby.len(), 1);
    }

    #[test]
    fn first_sighting_of_a_timestamp_older_than_lease_plus_margin_is_dropped_at_once() {
        let now = time::UnixTime::from_secs(1000);
        let member = discovery::RoomMember {
            updated_at: time::UnixTime::EPOCH,
            ..testing::room_member("p", None, discovery::RoomMembership::HistoryHolder, &[])
        };
        let mut lobby = testing::lobby_with(vec![(
            "r",
            testing::room(discovery::RoomSize::DEFAULT, 0, vec![member]),
        )]);
        drop_members_whose_lease_lapsed(
            &testing::lease_limits(Duration::from_secs(90), Duration::from_secs(600)),
            &mut lobby,
            &BTreeMap::new(),
            now,
        );
        assert!(lobby.is_empty());
    }
}
