use atomic_delegate_macros::atomic_delegates;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::direct_dial;
use crate::identity;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref, DerefMut)]
pub struct PeerBook(pub Vec<direct_dial::KnownPeer>);

impl TryFrom<&std::path::Path> for PeerBook {
    type Error = direct_dial::PeerBookError;

    fn try_from(path: &std::path::Path) -> Result<Self, Self::Error> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }
}

#[atomic_delegates]
impl PeerBook {
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), direct_dial::PeerBookError> {}
    pub fn quarantine_corrupt_file(path: &std::path::Path) -> Option<std::path::PathBuf> {}
    pub fn update_peer_addrs(&mut self, peer: &identity::Peer) -> bool {}
}

impl PeerBook {
    pub fn add_friend(&mut self, peer: identity::Peer) -> bool {
        if self.iter().any(|entry| entry.peer.id == peer.id) {
            return false;
        }
        self.push(direct_dial::KnownPeer {
            peer,
            kind: direct_dial::KnownPeerKind::Friend,
        });
        true
    }
    pub fn remove_friend(&mut self, id: &identity::PeerId) -> bool {
        let before = self.len();
        self.retain(|entry| entry.peer.id != *id);
        self.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::PeerBook;
    use crate::direct_dial;
    use crate::identity;

    fn friend_peer(name: &str) -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(name),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }
    }

    #[test]
    fn test_usage() {
        let mut book = PeerBook::default();
        assert!(book.add_friend(friend_peer("alice")));
        assert!(!book.add_friend(friend_peer("alice")));
        assert!(book.remove_friend(&identity::PeerId::from("alice")));
        assert!(!book.remove_friend(&identity::PeerId::from("alice")));
    }

    #[test]
    fn same_addresses_are_not_an_update() {
        let mut book = PeerBook::default();
        let peer = friend_peer("alice");
        assert!(book.add_friend(peer.clone()));
        assert!(!book.update_peer_addrs(&peer));
    }

    #[test]
    fn new_addresses_replace_the_old_ones() {
        let mut book = PeerBook::default();
        assert!(book.add_friend(friend_peer("alice")));
        let refreshed = identity::Peer {
            id: identity::PeerId::from("alice"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
        };
        assert!(book.update_peer_addrs(&refreshed));
        assert_eq!(book[0].peer.addrs, refreshed.addrs);
    }

    #[test]
    fn updating_a_stranger_reports_no_change() {
        let mut book = PeerBook::default();
        assert!(!book.update_peer_addrs(&friend_peer("mallory")));
    }

    #[test]
    fn missing_file_loads_an_empty_book() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.json");
        let book = PeerBook::try_from(path.as_path()).expect("missing is empty");
        assert!(book.is_empty());
    }

    #[test]
    fn corrupt_file_returns_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        std::fs::write(&path, b"not json").expect("write");
        let result = PeerBook::try_from(path.as_path());
        assert!(result.is_err());
    }

    #[test]
    fn round_trip_keeps_friends_and_kinds() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        let mut book = PeerBook::default();
        assert!(book.add_friend(friend_peer("alice")));
        book.push(direct_dial::KnownPeer {
            peer: friend_peer("bob"),
            kind: direct_dial::KnownPeerKind::PreviouslyConnected,
        });
        book.save_to_file(path.as_path()).expect("save");
        let loaded = PeerBook::try_from(path.as_path()).expect("load");
        assert_eq!(loaded, book);
    }
}
