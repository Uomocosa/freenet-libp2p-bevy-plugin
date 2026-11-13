use std::time::Duration;

use crate::discovery;
use crate::freenet;
use crate::identity;
use crate::p2p;
use crate::room_sync;
use crate::time;
use bevy::prelude::Resource;
use discovery::state_machine::{Handshake, HeartbeatTarget, Input, ObservedLobbyHeartbeat, Output};
use std::collections::{BTreeMap, BTreeSet};

use atomic_delegate_macros::atomic_delegates;

#[derive(Resource)]
pub struct DiscoveryState {
    pub me: identity::Peer,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub candidates: BTreeMap<identity::PeerId, discovery::state_machine::Candidate>,
    pub redial_after: BTreeMap<identity::PeerId, time::Deadline>,
    pub next_handshake: time::Deadline,
    pub freenet_peers: Vec<freenet::FreenetPeer>,
    pub is_lobby_fetched: bool,
    pub display_name: identity::Username,
    pub game: identity::GameId,
    pub held_rooms: BTreeMap<identity::RoomName, discovery::state_machine::HeldRoomHistory>,
    pub evicted_rooms: BTreeMap<identity::RoomName, time::UnixTime>,
    pub history_size_limits: discovery::HistorySizeLimits,
    pub lease_limits: discovery::LobbyLeaseLimits,
    pub observed_lobby_heartbeats:
        BTreeMap<(identity::RoomName, identity::PeerId), ObservedLobbyHeartbeat>,
    pub is_quitting: bool,
}

impl DiscoveryState {
    #[must_use]
    pub fn from_ready_peer(ready: identity::Peer, config: &discovery::Config) -> Self {
        let mut state = Self::new(identity::Peer {
            addrs: identity::PeerAddr::dialable(ready.addrs),
            id: ready.id,
        });
        state.apply_config(config);
        state
    }
    #[must_use]
    pub fn new(me: identity::Peer) -> Self {
        let lease = discovery::HolderHeartbeatIntervals::default();
        Self {
            me,
            lobby: discovery::Lobby(BTreeMap::new()),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            redial_after: BTreeMap::new(),
            next_handshake: time::Deadline::at(time::UnixTime::EPOCH),
            freenet_peers: Vec::new(),
            is_lobby_fetched: false,
            display_name: identity::Username(String::new()),
            game: discovery::Config::default().game,
            held_rooms: BTreeMap::new(),
            evicted_rooms: BTreeMap::new(),
            history_size_limits: discovery::HistorySizeLimits::default(),
            lease_limits: discovery::LobbyLeaseLimits {
                lease_ttl: lease.ttl,
                remote_clock_skew_margin: lease.remote_clock_skew_margin,
            },
            observed_lobby_heartbeats: BTreeMap::new(),
            is_quitting: false,
        }
    }
}

#[rustfmt::skip]
impl lele_bevy_state_machine::traits::StateMachine for DiscoveryState {
    type Input = Input;
    type Output = Output;
    type Now = time::UnixTime;
    fn update(&mut self, input: Self::Input, now: Self::Now) -> Vec<Self::Output> {
        match input {
            Input::Command(command) => self.handle_command(command, now),
            Input::Libp2pEvent(event) => self.handle_libp2p_event(event, now),
            Input::LobbyUpdated(lobby) => {
                self.is_lobby_fetched = true;
                self.handle_lobby(lobby, now)
            }
            Input::FreenetPeers(peers) => {
                self.freenet_peers = peers;
                Vec::new()
            }
            Input::OwnAddrsChanged(addrs) => {
                self.me.addrs = addrs;
                Vec::new()
            }
        }
    }
}

#[rustfmt::skip]
impl lele_bevy_state_machine::traits::PeriodicStateMachine for DiscoveryState {
    const TICK_INTERVAL: Duration = discovery::TICK;
    fn tick(&mut self, now: Self::Now) -> Vec<Self::Output> { Self::tick(self, now) }
}

#[atomic_delegates]
impl DiscoveryState {
    pub fn add_candidates(
        &mut self,
        peers: impl IntoIterator<Item = identity::Peer>,
        now: time::UnixTime,
    ) {
    }
    pub fn align_room_created_at(&mut self) {}
    pub fn broadcast_handshake(&self) -> Vec<Output> {}
    pub fn collect_room_peers_from_lobby(&mut self) -> Vec<Output> {}
    pub fn dial_candidates(&mut self, now: time::UnixTime) -> Vec<Output> {}
    pub fn drop_expired_candidates(&mut self, now: time::UnixTime) {}
    pub fn drop_lobby_members_whose_lease_lapsed(&mut self, now: time::UnixTime) -> Vec<Output> {}
    pub fn drop_member_who_left_room(&mut self, peer: &identity::PeerId) -> Vec<Output> {}
    pub fn drop_members_past_grace(&mut self, now: time::UnixTime) -> Vec<Output> {}
    pub fn evict_held_rooms_over_history_limit(
        &mut self,
        bytes_by_prefix: &BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount>,
    ) -> Vec<Output> {
    }
    pub fn handle_handshake(
        &mut self,
        from: &identity::PeerId,
        data: &[u8],
        now: time::UnixTime,
    ) -> Vec<Output> {
    }
    pub fn handle_lobby(&mut self, lobby: discovery::Lobby, now: time::UnixTime) -> Vec<Output> {}
    pub fn handshake(&self) -> Handshake {}
    pub fn heartbeat_targets(&self) -> Vec<HeartbeatTarget> {}
    pub fn join(
        &mut self,
        room: identity::RoomName,
        how_entered: discovery::HowRoomWasEntered,
        now: time::UnixTime,
    ) -> Vec<Output> {
    }
    pub fn leave(&mut self, now: time::UnixTime) -> Vec<Output> {}
    pub fn lobby_rows(&self, friend_ids: &[identity::PeerId]) -> Vec<discovery::LobbyRow> {}
    pub fn oldest_held_rooms_to_evict_until_history_fits(
        &self,
        history_bytes_by_held_room: &BTreeMap<identity::RoomName, p2p::HistoryByteCount>,
        current_room_history_bytes: p2p::HistoryByteCount,
    ) -> Vec<identity::RoomName> {
    }
    pub fn occupants(&self) -> Vec<identity::PeerId> {}
    pub fn record_lobby_heartbeats(&mut self, lobby: &discovery::Lobby, now: time::UnixTime) {}
    pub fn room_roster(&self, is_swarm_ready: bool) -> discovery::RoomRoster {}
    pub fn send_handshake(&self, peer: &identity::PeerId) -> Vec<Output> {}
    pub fn tick(&mut self, now: time::UnixTime) -> Vec<Output> {}
    pub fn wait_for_contract_self_heal(&mut self, now: time::UnixTime) -> Vec<Output> {}
}

impl DiscoveryState {
    #[must_use]
    pub fn peer_connection_status(&self, peer: &identity::PeerId) -> p2p::PeerConnectionStatus {
        if self.connected.contains(peer) {
            p2p::PeerConnectionStatus::Connected
        } else {
            p2p::PeerConnectionStatus::NotConnected
        }
    }

    pub fn connected_members(&self) -> impl Iterator<Item = &identity::PeerId> {
        let room = self.room.as_ref();
        self.connected
            .iter()
            .filter(move |peer| room.is_some_and(|room| room.members.contains_key(*peer)))
    }

    pub fn handle_command(
        &mut self,
        command: discovery::Command,
        now: time::UnixTime,
    ) -> Vec<Output> {
        let (room, how_entered) = match command {
            discovery::Command::Create(room) => (room, discovery::HowRoomWasEntered::Created),
            discovery::Command::Join(room) => (room, discovery::HowRoomWasEntered::Joined),
            discovery::Command::Leave => return self.leave(now),
            discovery::Command::Quit => return self.quit(now),
        };
        if room.as_str().trim().is_empty() {
            tracing::error!(target: "discovery", "discovery rejected: empty room name");
            Vec::new()
        } else {
            self.join(room, how_entered, now)
        }
    }
    pub fn handle_libp2p_event(
        &mut self,
        event: p2p::Libp2pEvent,
        now: time::UnixTime,
    ) -> Vec<Output> {
        let mut outputs = match event {
            p2p::Libp2pEvent::PeerConnected(peer) => {
                self.connected.insert(peer.clone());
                self.send_handshake(&peer)
            }
            p2p::Libp2pEvent::PeerDisconnected(peer) => {
                self.connected.remove(&peer);
                self.refresh_grace(&peer, now)
            }
            p2p::Libp2pEvent::PeerExchange { from, data } => {
                self.handle_handshake(&from, &data, now)
            }
            p2p::Libp2pEvent::HistoryRecordStored { .. } => {
                return vec![Output::Libp2pCommand(
                    p2p::Libp2pCommand::MeasureHistoryStore,
                )];
            }
            p2p::Libp2pEvent::HistoryStoreMeasured { bytes_by_prefix } => {
                return self.evict_held_rooms_over_history_limit(&bytes_by_prefix);
            }
            _ => return Vec::new(),
        };
        outputs.extend(self.dial_candidates(now));
        outputs
    }
    pub(crate) fn quit(&mut self, now: time::UnixTime) -> Vec<Output> {
        let outputs = self.leave(now);
        self.is_quitting = true;
        outputs
    }
    pub(crate) fn refresh_grace(
        &mut self,
        peer: &identity::PeerId,
        now: time::UnixTime,
    ) -> Vec<Output> {
        let Some(member) = self
            .room
            .as_mut()
            .and_then(|room| room.members.get_mut(peer))
        else {
            return Vec::new();
        };
        member.updated_at = now;
        vec![Output::Notify(discovery::Event::RoomMembersUpdated)]
    }
}

#[atomic_delegates]
impl DiscoveryState {
    pub(crate) fn apply_config(&mut self, config: &discovery::Config) {}
    pub(crate) fn desired_membership(
        state: &Self,
        room_entity_by_name: &BTreeMap<identity::RoomName, bevy::prelude::Entity>,
    ) -> (
        BTreeMap<identity::PeerId, bevy::prelude::Entity>,
        BTreeMap<identity::PeerId, BTreeSet<bevy::prelude::Entity>>,
    ) {
    }
    pub(crate) fn desired_rooms(state: &Self) -> BTreeSet<identity::RoomName> {}
    pub(crate) fn room_prefix_and_lobby_status(
        state: Option<&Self>,
        game: &identity::GameId,
        room_name: &identity::RoomName,
    ) -> (
        p2p::HistoryKeyPrefix,
        room_sync::state_machine::RoomLiveness,
    ) {
    }
}

#[cfg(test)]
mod tests {

    use super::DiscoveryState;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::time;
    use discovery::state_machine::{Input, Output};
    use lele_bevy_state_machine::traits::StateMachine;

    #[test]
    fn test_usage() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        assert!(state.room.is_none());
    }

    #[test]
    fn test_usage_1() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let peers = vec![
            identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: vec![identity::PeerAddr::from("/ip4/1")],
            },
            identity::Peer {
                id: identity::PeerId::from("a"),
                addrs: vec![identity::PeerAddr::from("/ip4/2")],
            },
            identity::Peer {
                id: identity::PeerId::from("b"),
                addrs: Vec::new(),
            },
        ];
        state.add_candidates(peers, time::UnixTime::from_secs(1));
        assert_eq!(state.candidates.len(), 1);
        assert!(state.candidates.contains_key(&identity::PeerId::from("a")));
    }

    #[test]
    fn test_usage_2() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.connected.insert(identity::PeerId::from("a"));
        let room = identity::RoomName::from("r");
        let mut outputs = state.handle_command(
            discovery::Command::Create(room),
            time::UnixTime::from_secs(1),
        );
        assert!(state.room.is_some());
        outputs
            .extend(state.handle_command(discovery::Command::Leave, time::UnixTime::from_secs(2)));
        assert!(state.room.is_none());
        assert!(matches!(
            outputs.as_slice(),
            [
                Output::Libp2pCommand(p2p::Libp2pCommand::PeerExchange { .. }),
                Output::Notify(discovery::Event::Joined(_)),
                Output::Libp2pCommand(p2p::Libp2pCommand::PeerExchange { .. }),
                Output::Notify(discovery::Event::Left(_)),
            ]
        ));
    }

    #[test]
    fn test_empty_room_rejected() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let outputs = state.handle_command(
            discovery::Command::Create(identity::RoomName::from("   ")),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(outputs.len(), 0);
        assert!(state.room.is_none());
    }

    #[test]
    fn test_usage_3() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let outputs = state.handle_libp2p_event(
            p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("a")),
            time::UnixTime::from_secs(1),
        );
        assert!(state.connected.contains(&identity::PeerId::from("a")));
        assert!(matches!(
            outputs.as_slice(),
            [Output::Libp2pCommand(p2p::Libp2pCommand::PeerExchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
        let _ = state.handle_libp2p_event(
            p2p::Libp2pEvent::PeerDisconnected(identity::PeerId::from("a")),
            time::UnixTime::from_secs(2),
        );
        assert!(state.connected.is_empty());
    }

    #[test]
    fn test_usage_4() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        assert_eq!(state.me.id, identity::PeerId::from("me"));
        assert!(state.room.is_none());
        assert!(state.lobby.is_empty());
        assert!(!state.is_lobby_fetched);
    }

    #[test]
    fn test_usage_5() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let room = identity::RoomName::from("r");
        let outputs = state.update(
            Input::Command(discovery::Command::Join(room.clone())),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::Joined(room))]
        );
    }

    #[test]
    fn own_addrs_changed_replaces_own_addrs() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/1")],
        });
        let addrs = vec![identity::PeerAddr::from("/ip4/2")];
        state.update(
            Input::OwnAddrsChanged(addrs.clone()),
            time::UnixTime::from_secs(1),
        );
        assert_eq!(state.me.addrs, addrs);
    }
}
