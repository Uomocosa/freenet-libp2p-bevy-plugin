use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::broadcast;

use crate::p2p;

const LIBP2P_EVENT_CAPACITY: usize = 1024;

#[derive(Resource, Deref)]
pub struct Libp2pEventChannel {
    pub tx: broadcast::Sender<p2p::Libp2pEvent>,
}

impl Default for Libp2pEventChannel {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(LIBP2P_EVENT_CAPACITY);
        Self { tx }
    }
}

#[rustfmt::skip]
impl Libp2pEventChannel {
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<p2p::Libp2pEvent> { self.tx.subscribe() }
    pub fn send(&self, event: p2p::Libp2pEvent) { self.tx.send(event).ok(); }
}

#[cfg(test)]
mod tests {
    use super::Libp2pEventChannel;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let tap = Libp2pEventChannel::default();
        let mut first = tap.subscribe();
        let mut second = tap.subscribe();
        tap.send(p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("p")));
        let expected = p2p::Libp2pEvent::PeerConnected(identity::PeerId::from("p"));
        assert_eq!(first.try_recv(), Ok(expected.clone()));
        assert_eq!(second.try_recv(), Ok(expected));
    }
}
