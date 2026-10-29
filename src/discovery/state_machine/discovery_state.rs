use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{Handshake, HeartbeatTarget, Input, ObservedLobbyHeartbeat, Output};
use std::collections::{BTreeMap, BTreeSet};

use atomic_delegate_macros::atomic_delegates;

pub struct DiscoveryState {
    pub me: identity::Peer,
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub candidates: BTreeMap<identity::PeerId, discovery::state_machine::Candidate>,
    pub last_dialed_at: BTreeMap<identity::PeerId, discovery::UnixTime>,
    pub last_handshake_at: Option<discovery::UnixTime>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub is_lobby_attached: bool,
    pub display_name: identity::Username,
    pub game: identity::GameId,
    pub held_rooms: BTreeMap<identity::RoomName, discovery::state_machine::HeldRoomHistory>,
    pub evicted_rooms: BTreeMap<identity::RoomName, discovery::UnixTime>,
    pub history_size_limits: discovery::HistorySizeLimits,
    pub lease_limits: discovery::LobbyLeaseLimits,
    pub observed_lobby_heartbeats:
        BTreeMap<(identity::RoomName, identity::PeerId), ObservedLobbyHeartbeat>,
    pub is_quitting: bool,
}

impl DiscoveryState {
    #[must_use]
    pub fn new(me: identity::Peer) -> Self {
        let lease = discovery::HolderHeartbeatIntervals::default();
        Self {
            me,
            lobby: discovery::Lobby(BTreeMap::new()),
            room: None,
            connected: BTreeSet::new(),
            candidates: BTreeMap::new(),
            last_dialed_at: BTreeMap::new(),
            last_handshake_at: None,
            freenet_peers: Vec::new(),
            is_lobby_attached: false,
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

#[atomic_delegates]
impl DiscoveryState {
    pub fn align_room_created_at(&mut self) {}
    pub fn broadcast_handshake(&self) -> Vec<Output> {}
    pub fn collect_room_peers_from_lobby(&mut self) -> Vec<Output> {}
    pub fn dial_candidates(&mut self, now: discovery::UnixTime) -> Vec<Output> {}
    pub fn drop_lobby_members_whose_lease_lapsed(
        &mut self,
        now: discovery::UnixTime,
    ) -> Vec<Output> {
    }
    pub fn drop_member(&mut self, peer: &identity::PeerId) -> Vec<Output> {}
    pub fn drop_oldest_history(
        &mut self,
        bytes_by_prefix: &BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount>,
    ) -> Vec<Output> {
    }
    pub fn held_rooms_to_evict_until_history_fits(
        &self,
        held_bytes: &BTreeMap<identity::RoomName, p2p::HistoryByteCount>,
        current_room_bytes: p2p::HistoryByteCount,
        limit: p2p::HistoryByteCount,
    ) -> Vec<identity::RoomName> {
    }
    pub fn handle_handshake(
        &mut self,
        from: &identity::PeerId,
        data: &[u8],
        now: discovery::UnixTime,
    ) -> Vec<Output> {
    }
    pub fn handle_lobby(
        &mut self,
        lobby: discovery::Lobby,
        now: discovery::UnixTime,
    ) -> Vec<Output> {
    }
    pub fn handshake(&self) -> Handshake {}
    pub fn heartbeat_targets(&self) -> Vec<HeartbeatTarget> {}
    pub fn join(
        &mut self,
        room: identity::RoomName,
        entry: discovery::RoomEntry,
        now: discovery::UnixTime,
    ) -> Vec<Output> {
    }
    pub fn leave(&mut self, now: discovery::UnixTime) -> Vec<Output> {}
    pub fn prune_members(&mut self, now: discovery::UnixTime) -> Vec<Output> {}
    pub fn record_lobby_heartbeats(&mut self, lobby: &discovery::Lobby, now: discovery::UnixTime) {}
    pub fn send_handshake(&self, peer: &identity::PeerId) -> Vec<Output> {}
    pub fn tick(&mut self, now: discovery::UnixTime) -> Vec<Output> {}
    pub fn wait_for_contract_self_heal(&mut self, now: discovery::UnixTime) -> Vec<Output> {}
}

#[rustfmt::skip]
impl DiscoveryState {
    pub fn add_candidates(
        &mut self,
        peers: impl IntoIterator<Item = identity::Peer>,
        now: discovery::UnixTime,
    ) {
        for peer in peers {
            if peer.id == self.me.id || peer.id.is_empty() || peer.addrs.is_empty() {
                continue;
            }
            self.candidates.insert(
                peer.id,
                discovery::state_machine::Candidate {
                    addrs: peer.addrs,
                    seen_at: now,
                },
            );
        }
    }
    pub fn handle_command(
        &mut self,
        command: discovery::Command,
        now: discovery::UnixTime,
    ) -> Vec<Output> {
        let (room, entry) = match command {
            discovery::Command::Create(room) => (room, discovery::RoomEntry::Created),
            discovery::Command::Join(room) => (room, discovery::RoomEntry::Joined),
            discovery::Command::Leave => return self.leave(now),
            discovery::Command::Quit => return self.quit(now),
        };
        if room.as_str().trim().is_empty() {
            tracing::error!(target: "discovery", "discovery rejected: empty room name");
            Vec::new()
        } else {
            self.join(room, entry, now)
        }
    }
    pub fn handle_net_event(
        &mut self,
        event: p2p::NetEvent,
        now: discovery::UnixTime,
    ) -> Vec<Output> {
        let mut outputs = match event {
            p2p::NetEvent::PeerConnected(peer) => {
                self.connected.insert(peer.clone());
                self.send_handshake(&peer)
            }
            p2p::NetEvent::PeerDisconnected(peer) => {
                self.connected.remove(&peer);
                self.refresh_grace(&peer, now)
            }
            p2p::NetEvent::PeerExchange { from, data } => {
                self.handle_handshake(&from, &data, now)
            }
            p2p::NetEvent::HistoryRecordStored { .. } => {
                return vec![Output::NetCommand(p2p::NetCommand::MeasureHistoryStore)];
            }
            p2p::NetEvent::HistoryStoreMeasured { bytes_by_prefix } => {
                return self.drop_oldest_history(&bytes_by_prefix);
            }
            _ => return Vec::new(),
        };
        outputs.extend(self.dial_candidates(now));
        outputs
    }
    #[must_use]
    pub fn status(&self) -> discovery::Status {
        discovery::Status {
            lobby: self.lobby.clone(),
            room: self.room.clone(),
            connected: self.connected.clone(),
            freenet_peers: self.freenet_peers.clone(),
            is_lobby_attached: self.is_lobby_attached,
            self_peer_id: Some(self.me.id.clone()),
        }
    }
    pub fn update(&mut self, input: Input, now: discovery::UnixTime) -> Vec<Output> {
        match input {
            Input::Command(command) => self.handle_command(command, now),
            Input::NetEvent(event) => self.handle_net_event(event, now),
            Input::LobbyUpdated(lobby) => {
                self.is_lobby_attached = true;
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
            Input::Tick => self.tick(now),
        }
    }
    pub(crate) fn quit(&mut self, now: discovery::UnixTime) -> Vec<Output> {
        let outputs = self.leave(now);
        self.is_quitting = true;
        outputs
    }
    pub(crate) fn refresh_grace(
        &mut self,
        peer: &identity::PeerId,
        now: discovery::UnixTime,
    ) -> Vec<Output> {
        let Some(member) = self
            .room
            .as_mut()
            .and_then(|room| room.members.get_mut(peer))
        else {
            return Vec::new();
        };
        member.updated_at = now;
        vec![Output::Notify(discovery::Event::MembersChanged)]
    }
}

#[cfg(test)]
mod tests {

    use super::DiscoveryState;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{Input, Output};

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
        state.add_candidates(peers, discovery::UnixTime::from(1));
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
            discovery::UnixTime::from(1),
        );
        assert!(state.room.is_some());
        outputs
            .extend(state.handle_command(discovery::Command::Leave, discovery::UnixTime::from(2)));
        assert!(state.room.is_none());
        assert!(matches!(
            outputs.as_slice(),
            [
                Output::NetCommand(p2p::NetCommand::PeerExchange { .. }),
                Output::Notify(discovery::Event::Joined(_)),
                Output::NetCommand(p2p::NetCommand::PeerExchange { .. }),
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
            discovery::UnixTime::from(1),
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
        let outputs = state.handle_net_event(
            p2p::NetEvent::PeerConnected(identity::PeerId::from("a")),
            discovery::UnixTime::from(1),
        );
        assert!(state.connected.contains(&identity::PeerId::from("a")));
        assert!(matches!(
            outputs.as_slice(),
            [Output::NetCommand(p2p::NetCommand::PeerExchange { peer_id, .. })] if peer_id.as_str() == "a"
        ));
        let _ = state.handle_net_event(
            p2p::NetEvent::PeerDisconnected(identity::PeerId::from("a")),
            discovery::UnixTime::from(2),
        );
        assert!(state.connected.is_empty());
    }

    #[test]
    fn test_usage_4() {
        let state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        let expected = discovery::Status {
            self_peer_id: Some(identity::PeerId::from("me")),
            ..discovery::Status::default()
        };
        assert_eq!(state.status(), expected);
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
            discovery::UnixTime::from(1),
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
            discovery::UnixTime::from(1),
        );
        assert_eq!(state.me.addrs, addrs);
    }
}
