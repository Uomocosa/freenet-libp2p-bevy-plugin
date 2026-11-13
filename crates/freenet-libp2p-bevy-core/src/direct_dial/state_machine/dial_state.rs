use std::collections::BTreeSet;
use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;

use crate::direct_dial;
use crate::identity;
use crate::time;

#[derive(Resource, Debug, Default, Clone)]
pub struct DialState {
    pub entries: Vec<direct_dial::state_machine::DialKnownPeer>,
    pub connected: BTreeSet<identity::PeerId>,
    pub dial_in_flight: BTreeSet<identity::PeerId>,
}

#[atomic_delegates]
impl DialState {
    pub fn from_peer_book(book: &direct_dial::PeerBook, now: time::UnixTime) -> Self {}
}

impl lele_bevy_state_machine::traits::StateMachine for DialState {
    type Input = direct_dial::state_machine::Input;
    type Output = direct_dial::state_machine::Output;
    type Now = time::UnixTime;
    fn update(&mut self, input: Self::Input, now: Self::Now) -> Vec<Self::Output> {
        match input {
            direct_dial::state_machine::Input::FriendAdded(peer) => {
                self.handle_friend_added(peer, now)
            }
            direct_dial::state_machine::Input::FriendRemoved(id) => self.handle_friend_removed(&id),
            direct_dial::state_machine::Input::PeerConnected(id) => {
                self.handle_peer_connected(&id, now)
            }
            direct_dial::state_machine::Input::PeerDisconnected(id) => {
                self.handle_peer_disconnected(&id, now)
            }
            direct_dial::state_machine::Input::DialFailed(id) => self.handle_dial_failed(&id, now),
            direct_dial::state_machine::Input::PeerListenAddrs(peer) => {
                self.handle_peer_listen_addrs(&peer, now)
            }
        }
    }
}

#[rustfmt::skip]
impl lele_bevy_state_machine::traits::PeriodicStateMachine for DialState {
    const TICK_INTERVAL: Duration = direct_dial::state_machine::TICK_INTERVAL;
    fn tick(&mut self, now: Self::Now) -> Vec<Self::Output> { self.dial_due(now) }
}

#[atomic_delegates]
impl DialState {
    #[must_use]
    pub fn is_friend(&self, id: &identity::PeerId) -> bool {}
}

impl DialState {
    #[must_use]
    pub fn to_peer_book(&self) -> direct_dial::PeerBook {
        let mut book = direct_dial::PeerBook::default();
        for entry in &self.entries {
            book.push(direct_dial::KnownPeer {
                peer: entry.peer.clone(),
                kind: entry.kind,
            });
        }
        book
    }
}

#[atomic_delegates]
impl DialState {
    pub(crate) fn dial_due(
        &mut self,
        now: time::UnixTime,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn handle_friend_added(
        &mut self,
        peer: identity::Peer,
        now: time::UnixTime,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn handle_friend_removed(
        &mut self,
        id: &identity::PeerId,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn handle_peer_connected(
        &mut self,
        id: &identity::PeerId,
        now: time::UnixTime,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn handle_peer_disconnected(
        &mut self,
        id: &identity::PeerId,
        now: time::UnixTime,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn handle_dial_failed(
        &mut self,
        id: &identity::PeerId,
        now: time::UnixTime,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn handle_peer_listen_addrs(
        &mut self,
        peer: &identity::Peer,
        now: time::UnixTime,
    ) -> Vec<direct_dial::state_machine::Output> {
    }
    pub(crate) fn backoff_for(
        &self,
        kind: direct_dial::KnownPeerKind,
        failed_attempts: u32,
    ) -> std::time::Duration {
    }
}

#[cfg(test)]
mod tests {
    use lele_bevy_state_machine::traits::{PeriodicStateMachine, StateMachine};

    use super::DialState;
    use crate::direct_dial;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let state = DialState::default();
        assert_eq!(state.entries, Vec::new());
        assert!(state.connected.is_empty());
        assert!(state.dial_in_flight.is_empty());
    }

    fn friend_peer(name: &str) -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(name),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }
    }

    fn fresh_state() -> DialState {
        DialState::default()
    }

    #[test]
    fn update_dispatches_friend_added() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let outputs = state.update(
            direct_dial::state_machine::Input::FriendAdded(friend_peer("alice")),
            now,
        );
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn backoff_grows_and_caps() {
        let state = fresh_state();
        let first = state.backoff_for(direct_dial::KnownPeerKind::Friend, 0);
        let second = state.backoff_for(direct_dial::KnownPeerKind::Friend, 1);
        let capped = state.backoff_for(direct_dial::KnownPeerKind::Friend, 100);
        assert!(second > first);
        assert_eq!(capped, direct_dial::state_machine::FRIEND_MAX_BACKOFF);
        assert_eq!(first, direct_dial::state_machine::FRIEND_INITIAL_BACKOFF);
    }

    #[test]
    fn friend_never_dropped() {
        let mut state = fresh_state();
        let mut now = time::UnixTime::from_secs(10);
        let peer = friend_peer("alice");
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(peer.clone()),
            now,
        );
        for _ in 0..10 {
            state.dial_in_flight.remove(&peer.id);
            now = now.saturating_add(std::time::Duration::from_secs(1000));
            let outputs = state.update(
                direct_dial::state_machine::Input::DialFailed(peer.id.clone()),
                now,
            );
            assert!(outputs.iter().all(|output| !matches!(
                output,
                direct_dial::state_machine::Output::SavePeerBook(_)
            )));
        }
        assert!(state.entries.iter().any(|entry| entry.peer.id == peer.id));
    }

    #[test]
    fn known_peer_dropped_after_limit_with_save() {
        let mut state = fresh_state();
        let mut now = time::UnixTime::from_secs(10);
        let id = identity::PeerId::from("bob");
        let _ = state.update(
            direct_dial::state_machine::Input::PeerConnected(id.clone()),
            now,
        );
        let peer = identity::Peer {
            id: id.clone(),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4002")],
        };
        let _ = state.update(
            direct_dial::state_machine::Input::PeerListenAddrs(peer),
            now,
        );
        assert!(state.entries.iter().any(|entry| entry.peer.id == id));
        let _ = state.update(
            direct_dial::state_machine::Input::PeerDisconnected(id.clone()),
            now,
        );
        let mut saved = false;
        for _ in 0..direct_dial::state_machine::KNOWN_MAX_FAILURES {
            state.dial_in_flight.remove(&id);
            now = now.saturating_add(std::time::Duration::from_secs(1000));
            let outputs = state.update(
                direct_dial::state_machine::Input::DialFailed(id.clone()),
                now,
            );
            if outputs
                .iter()
                .any(|output| matches!(output, direct_dial::state_machine::Output::SavePeerBook(_)))
            {
                saved = true;
            }
        }
        assert!(saved);
        assert!(!state.entries.iter().any(|entry| entry.peer.id == id));
    }

    #[test]
    fn no_dial_while_connected_or_in_flight() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let peer = friend_peer("alice");
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(peer.clone()),
            now,
        );
        let _ = state.update(
            direct_dial::state_machine::Input::PeerConnected(peer.id.clone()),
            now,
        );
        let outputs = state.tick(now);
        assert_eq!(outputs, Vec::new());
        let _ = state.update(
            direct_dial::state_machine::Input::PeerDisconnected(peer.id.clone()),
            now,
        );
        state.dial_in_flight.insert(peer.id);
        let outputs = state.tick(now);
        assert_eq!(outputs, Vec::new());
    }

    #[test]
    fn disconnect_schedules_redial() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let peer = friend_peer("alice");
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(peer.clone()),
            now,
        );
        let _ = state.update(
            direct_dial::state_machine::Input::PeerConnected(peer.id.clone()),
            now,
        );
        let later = time::UnixTime::from_secs(20);
        let _ = state.update(
            direct_dial::state_machine::Input::PeerDisconnected(peer.id.clone()),
            later,
        );
        assert!(!state.connected.contains(&peer.id));
        let outputs = state.tick(time::UnixTime::from_secs(1000));
        assert!(outputs.iter().any(|output| matches!(
            output,
            direct_dial::state_machine::Output::Dial(dialed) if dialed.id == peer.id
        )));
    }

    #[test]
    fn connect_resets_failures() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let peer = friend_peer("alice");
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(peer.clone()),
            now,
        );
        state.dial_in_flight.remove(&peer.id);
        let _ = state.update(
            direct_dial::state_machine::Input::DialFailed(peer.id.clone()),
            now,
        );
        let attempts = state
            .entries
            .iter()
            .find(|entry| entry.peer.id == peer.id)
            .map(|entry| entry.failed_attempts)
            .unwrap_or_default();
        assert_eq!(attempts, 1);
        let _ = state.update(
            direct_dial::state_machine::Input::PeerConnected(peer.id.clone()),
            now,
        );
        let attempts = state
            .entries
            .iter()
            .find(|entry| entry.peer.id == peer.id)
            .map(|entry| entry.failed_attempts)
            .unwrap_or_default();
        assert_eq!(attempts, 0);
    }

    #[test]
    fn never_demotes_a_friend() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let peer = friend_peer("alice");
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(peer.clone()),
            now,
        );
        let _ = state.update(
            direct_dial::state_machine::Input::PeerConnected(peer.id.clone()),
            now,
        );
        let _ = state.update(
            direct_dial::state_machine::Input::PeerListenAddrs(peer.clone()),
            now,
        );
        let kind = state
            .entries
            .iter()
            .find(|entry| entry.peer.id == peer.id)
            .map(|entry| entry.kind)
            .expect("friend stays");
        assert_eq!(kind, direct_dial::KnownPeerKind::Friend);
    }

    #[test]
    fn removed_friend_no_longer_dials() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let peer = friend_peer("alice");
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(peer.clone()),
            now,
        );
        let _ = state.update(
            direct_dial::state_machine::Input::FriendRemoved(peer.id),
            now,
        );
        let outputs = state.tick(time::UnixTime::from_secs(1000));
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.entries, Vec::new());
    }

    #[test]
    fn saved_book_dials_on_start() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(friend_peer("alice")));
        let now = time::UnixTime::from_secs(10);
        let mut state = DialState::from_peer_book(&book, now);
        let outputs = state.tick(now);
        assert_eq!(outputs.len(), 1);
        assert!(matches!(
            outputs.first(),
            Some(direct_dial::state_machine::Output::Dial(dialed))
                if dialed.id == identity::PeerId::from("alice")
        ));
    }

    #[test]
    fn book_round_trips_through_state() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(friend_peer("alice")));
        let state = DialState::from_peer_book(&book, time::UnixTime::from_secs(1));
        assert_eq!(state.to_peer_book(), book);
    }

    #[test]
    fn friend_added_saves_the_book() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let outputs = state.update(
            direct_dial::state_machine::Input::FriendAdded(friend_peer("alice")),
            now,
        );
        assert!(
            outputs.iter().any(|output| matches!(
                output,
                direct_dial::state_machine::Output::SavePeerBook(_)
            ))
        );
    }

    #[test]
    fn friend_removed_saves_the_book() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(friend_peer("alice")),
            now,
        );
        let outputs = state.update(
            direct_dial::state_machine::Input::FriendRemoved(identity::PeerId::from("alice")),
            now,
        );
        assert!(
            outputs.iter().any(|output| matches!(
                output,
                direct_dial::state_machine::Output::SavePeerBook(_)
            ))
        );
        assert_eq!(state.entries, Vec::new());
    }

    #[test]
    fn is_friend_reports_friends_only() {
        let mut state = fresh_state();
        let now = time::UnixTime::from_secs(10);
        assert!(!state.is_friend(&identity::PeerId::from("alice")));
        let _ = state.update(
            direct_dial::state_machine::Input::FriendAdded(friend_peer("alice")),
            now,
        );
        assert!(state.is_friend(&identity::PeerId::from("alice")));
        let _ = state.update(
            direct_dial::state_machine::Input::FriendRemoved(identity::PeerId::from("alice")),
            now,
        );
        assert!(!state.is_friend(&identity::PeerId::from("alice")));
    }
}
