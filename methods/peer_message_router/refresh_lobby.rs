use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::testing;
use crate::time;

pub fn refresh_lobby(
    router: &mut testing::PeerMessageRouter,
    peer: &identity::PeerId,
    targets: &[discovery::state_machine::HeartbeatTarget],
    now: time::UnixTime,
) -> discovery::Lobby {
    router
        .heartbeat_targets
        .insert(peer.clone(), targets.to_owned());
    let mut fresh = discovery::Lobby::default();
    for target in targets {
        let record = fresh
            .entry(target.room.clone())
            .or_insert_with(|| discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: target.created_at,
                members: BTreeMap::new(),
            });
        if target.created_at < record.created_at {
            record.created_at = target.created_at;
        }
        record.members.insert(
            peer.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer.clone(),
                    addrs: target.addrs.clone(),
                },
                updated_at: now,
                display_name: Some(target.display_name.clone()),
                membership: target.membership,
            },
        );
    }
    fresh.drop_rooms_where_every_member_quit();
    router.shared_lobby.merge(fresh);
    router.shared_lobby.clone()
}
#[cfg(test)]
mod tests {
    use super::refresh_lobby;
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::time;

    fn heartbeat(room: &str, created_at: u64) -> discovery::state_machine::HeartbeatTarget {
        discovery::state_machine::HeartbeatTarget {
            room: identity::RoomName::from(room),
            created_at: time::UnixTime::from(created_at),
            addrs: vec![identity::PeerAddr::from("/ip4/1/tcp/4001")],
            display_name: identity::Username::from("someone"),
            membership: discovery::RoomMembership::Occupant,
        }
    }

    #[test]
    fn test_usage() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let first = identity::PeerId::from("a");
        let second = identity::PeerId::from("b");
        let lobby = refresh_lobby(
            &mut router,
            &first,
            &[heartbeat("room-a", 10)],
            time::UnixTime::from(20),
        );
        assert!(lobby.contains_key(&identity::RoomName::from("room-a")));
        let lobby = refresh_lobby(
            &mut router,
            &second,
            &[heartbeat("room-a", 10)],
            time::UnixTime::from(21),
        );
        let record = lobby
            .get(&identity::RoomName::from("room-a"))
            .expect("room-a is present");
        assert_eq!(record.members.len(), 2);
        assert!(record.members.contains_key(&first));
        assert!(record.members.contains_key(&second));
        assert_eq!(router.shared_lobby, lobby);
    }

    #[test]
    fn latest_targets_replace_earlier_ones() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = identity::PeerId::from("a");
        refresh_lobby(
            &mut router,
            &peer,
            &[heartbeat("room-a", 10)],
            time::UnixTime::from(20),
        );
        let lobby = refresh_lobby(&mut router, &peer, &[], time::UnixTime::from(21));
        let record = lobby
            .get(&identity::RoomName::from("room-a"))
            .expect("room-a stays until its lease lapses");
        assert_eq!(
            record.members.get(&peer).map(|member| member.updated_at),
            Some(time::UnixTime::from(20))
        );
    }

    #[test]
    fn peer_that_stops_heartbeating_keeps_its_old_updated_at() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let quiet = identity::PeerId::from("a");
        let live = identity::PeerId::from("b");
        refresh_lobby(
            &mut router,
            &quiet,
            &[heartbeat("room-a", 10)],
            time::UnixTime::from(20),
        );
        refresh_lobby(
            &mut router,
            &live,
            &[heartbeat("room-a", 10)],
            time::UnixTime::from(30),
        );
        let lobby = refresh_lobby(
            &mut router,
            &live,
            &[heartbeat("room-a", 10)],
            time::UnixTime::from(40),
        );
        let record = lobby
            .get(&identity::RoomName::from("room-a"))
            .expect("room-a is present");
        assert_eq!(
            record.members.get(&quiet).map(|member| member.updated_at),
            Some(time::UnixTime::from(20))
        );
        assert_eq!(
            record.members.get(&live).map(|member| member.updated_at),
            Some(time::UnixTime::from(40))
        );
    }

    #[test]
    fn rooms_where_every_member_quit_are_dropped() {
        let mut router = testing::PeerMessageRouter::new(identity::GameId::from("game"));
        let peer = identity::PeerId::from("a");
        let mut departed = heartbeat("room-a", 10);
        departed.membership = discovery::RoomMembership::Departed;
        let lobby = refresh_lobby(&mut router, &peer, &[departed], time::UnixTime::from(20));
        assert!(!lobby.contains_key(&identity::RoomName::from("room-a")));
    }
}
