use std::collections::{BTreeMap, HashMap};

use atomic_delegate_macros::atomic_delegates;

use crate::discovery;
use crate::identity;
use crate::testing;
use crate::time;

#[derive(Debug, Default)]
pub struct LobbyKit {
    pub(crate) shared_lobbies: HashMap<identity::GameId, discovery::Lobby>,
    pub(crate) heartbeat_targets: HashMap<
        identity::GameId,
        BTreeMap<identity::PeerId, Vec<discovery::state_machine::HeartbeatTarget>>,
    >,
}

#[rustfmt::skip]
impl testing::traits::FakeNetworkKit for LobbyKit {
    fn on_hello(&mut self, _peer: &identity::Peer, game: &identity::GameId, _now: time::UnixTime) -> Vec<Vec<u8>> { self.snapshot_frames(game) }
    fn on_extension(&mut self, from: &identity::PeerId, game: &identity::GameId, bytes: Vec<u8>, now: time::UnixTime) -> Vec<testing::FanOut> { self.apply_extension(from, game, &bytes, now) }
    fn on_disconnect(&mut self, peer: &identity::PeerId, game: &identity::GameId, _now: time::UnixTime) -> Vec<testing::FanOut> { self.apply_disconnect(peer, game) }
}

#[atomic_delegates]
impl LobbyKit {
    pub(crate) fn snapshot_frames(&self, game: &identity::GameId) -> Vec<Vec<u8>> {}
    pub(crate) fn apply_extension(
        &mut self,
        from: &identity::PeerId,
        game: &identity::GameId,
        bytes: &[u8],
        now: time::UnixTime,
    ) -> Vec<testing::FanOut> {
    }
    pub(crate) fn apply_disconnect(
        &mut self,
        peer: &identity::PeerId,
        game: &identity::GameId,
    ) -> Vec<testing::FanOut> {
    }
    pub(crate) fn refresh_heartbeats(
        &mut self,
        peer: &identity::PeerId,
        game: &identity::GameId,
        targets: &[discovery::state_machine::HeartbeatTarget],
        now: time::UnixTime,
    ) -> discovery::Lobby {
    }
}

#[cfg(test)]
mod tests {
    use super::LobbyKit;
    use crate::discovery;
    use crate::identity;
    use crate::testing;
    use crate::testing::traits::FakeNetworkKit;
    use crate::time;

    fn heartbeat(
        room: &str,
        created_at: u64,
        membership: discovery::RoomMembership,
    ) -> discovery::state_machine::HeartbeatTarget {
        discovery::state_machine::HeartbeatTarget {
            room: identity::RoomName::from(room),
            created_at: time::UnixTime::from_secs(created_at),
            addrs: vec![identity::PeerAddr::from("/ip4/1/tcp/4001")],
            display_name: identity::Username::from("someone"),
            membership,
        }
    }

    fn heartbeat_bytes(targets: Vec<discovery::state_machine::HeartbeatTarget>) -> Vec<u8> {
        bincode::serialize(&discovery::LobbyWire::HeartbeatTargets(targets)).expect("encode")
    }

    fn snapshot_of(frames: &[Vec<u8>]) -> discovery::Lobby {
        match bincode::deserialize::<discovery::LobbyWire>(&frames[0]).expect("decode") {
            discovery::LobbyWire::Snapshot(lobby) => lobby,
            discovery::LobbyWire::HeartbeatTargets(_) => panic!("expected a snapshot"),
        }
    }

    #[test]
    fn test_usage() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let peer = identity::Peer {
            id: identity::PeerId::from("a"),
            addrs: Vec::new(),
        };
        let frames = kit.on_hello(&peer, &game, time::UnixTime::from_secs(1));
        assert!(snapshot_of(&frames).is_empty());
    }

    #[test]
    fn peer_connecting_after_a_room_was_created_sees_it_with_no_further_heartbeat() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let first = identity::PeerId::from("a");
        kit.on_extension(
            &first,
            &game,
            heartbeat_bytes(vec![heartbeat(
                "room-a",
                10,
                discovery::RoomMembership::Occupant,
            )]),
            time::UnixTime::from_secs(20),
        );
        let late = identity::Peer {
            id: identity::PeerId::from("b"),
            addrs: Vec::new(),
        };
        let lobby = snapshot_of(&kit.on_hello(&late, &game, time::UnixTime::from_secs(30)));
        assert!(
            lobby.contains_key(&identity::RoomName::from("room-a")),
            "late joiner must see the room with no heartbeat of its own"
        );
    }

    #[test]
    fn heartbeats_fan_the_merged_lobby_out_to_the_whole_game() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let first = identity::PeerId::from("a");
        let second = identity::PeerId::from("b");
        kit.on_extension(
            &first,
            &game,
            heartbeat_bytes(vec![heartbeat(
                "room-a",
                10,
                discovery::RoomMembership::Occupant,
            )]),
            time::UnixTime::from_secs(20),
        );
        let fan_outs = kit.on_extension(
            &second,
            &game,
            heartbeat_bytes(vec![heartbeat(
                "room-a",
                10,
                discovery::RoomMembership::Occupant,
            )]),
            time::UnixTime::from_secs(21),
        );
        assert_eq!(fan_outs.len(), 1);
        assert_eq!(fan_outs[0].to, testing::FanOutTarget::Game);
        let record = snapshot_of(&[fan_outs[0].bytes.clone()]);
        let record = record
            .get(&identity::RoomName::from("room-a"))
            .expect("room-a is present");
        assert_eq!(record.members.len(), 2);
        assert!(record.members.contains_key(&first));
        assert!(record.members.contains_key(&second));
    }

    #[test]
    fn latest_targets_replace_earlier_ones() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let peer = identity::PeerId::from("a");
        kit.refresh_heartbeats(
            &peer,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Occupant)],
            time::UnixTime::from_secs(20),
        );
        let lobby = kit.refresh_heartbeats(&peer, &game, &[], time::UnixTime::from_secs(21));
        let record = lobby
            .get(&identity::RoomName::from("room-a"))
            .expect("room-a stays until its lease lapses");
        assert_eq!(
            record.members.get(&peer).map(|member| member.updated_at),
            Some(time::UnixTime::from_secs(20))
        );
    }

    #[test]
    fn peer_that_stops_heartbeating_keeps_its_old_updated_at() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let quiet = identity::PeerId::from("a");
        let live = identity::PeerId::from("b");
        kit.refresh_heartbeats(
            &quiet,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Occupant)],
            time::UnixTime::from_secs(20),
        );
        kit.refresh_heartbeats(
            &live,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Occupant)],
            time::UnixTime::from_secs(30),
        );
        let lobby = kit.refresh_heartbeats(
            &live,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Occupant)],
            time::UnixTime::from_secs(40),
        );
        let record = lobby
            .get(&identity::RoomName::from("room-a"))
            .expect("room-a is present");
        assert_eq!(
            record.members.get(&quiet).map(|member| member.updated_at),
            Some(time::UnixTime::from_secs(20))
        );
        assert_eq!(
            record.members.get(&live).map(|member| member.updated_at),
            Some(time::UnixTime::from_secs(40))
        );
    }

    #[test]
    fn rooms_where_every_member_quit_are_dropped() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let peer = identity::PeerId::from("a");
        let lobby = kit.refresh_heartbeats(
            &peer,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Departed)],
            time::UnixTime::from_secs(20),
        );
        assert!(!lobby.contains_key(&identity::RoomName::from("room-a")));
    }

    #[test]
    fn crashed_occupant_stays_listed_like_a_lease() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let holder = identity::PeerId::from("holder");
        kit.refresh_heartbeats(
            &holder,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Occupant)],
            time::UnixTime::from_secs(20),
        );
        kit.apply_disconnect(&holder, &game);
        assert!(
            kit.shared_lobbies
                .get(&game)
                .and_then(|lobby| lobby.get(&identity::RoomName::from("room-a")))
                .is_some_and(|record| record.members.contains_key(&holder)),
            "crashed holder stays listed so a joiner still sees the room"
        );
    }

    #[test]
    fn clean_quit_departed_is_removed() {
        let mut kit = LobbyKit::default();
        let game = identity::GameId::from("game");
        let quitter = identity::PeerId::from("quitter");
        kit.refresh_heartbeats(
            &quitter,
            &game,
            &[heartbeat("room-a", 10, discovery::RoomMembership::Departed)],
            time::UnixTime::from_secs(20),
        );
        kit.apply_disconnect(&quitter, &game);
        assert!(
            kit.shared_lobbies
                .get(&game)
                .and_then(|lobby| lobby.get(&identity::RoomName::from("room-a")))
                .is_none_or(|record| !record.members.contains_key(&quitter)),
            "clean quit removes the departed entry so the room can go"
        );
    }
}
