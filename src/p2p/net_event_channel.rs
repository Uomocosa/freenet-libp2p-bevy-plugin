use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::broadcast;

use crate::p2p;

const NET_EVENT_CAPACITY: usize = 1024;

#[derive(Resource, Deref)]
pub struct NetEventChannel {
    pub tx: broadcast::Sender<p2p::NetEvent>,
}

impl Default for NetEventChannel {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(NET_EVENT_CAPACITY);
        Self { tx }
    }
}

#[rustfmt::skip]
impl NetEventChannel {
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<p2p::NetEvent> { self.tx.subscribe() }
    pub fn send(&self, event: p2p::NetEvent) { self.tx.send(event).ok(); }
}

#[cfg(test)]
mod tests {
    use super::NetEventChannel;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let tap = NetEventChannel::default();
        let mut first = tap.subscribe();
        let mut second = tap.subscribe();
        tap.send(p2p::NetEvent::PeerConnected(identity::PeerId::from("p")));
        let expected = p2p::NetEvent::PeerConnected(identity::PeerId::from("p"));
        assert_eq!(first.try_recv(), Ok(expected.clone()));
        assert_eq!(second.try_recv(), Ok(expected));
    }
}
