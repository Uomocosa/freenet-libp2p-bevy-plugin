use bevy::prelude::Resource;
use tokio::sync::watch::{self, Receiver, Sender};

use crate::identity;
use identity::Peer;

#[derive(Resource)]
pub struct ReadyChannel {
    pub ready_tx: Sender<Option<Peer>>,
    pub ready_rx: Receiver<Option<Peer>>,
    pub observed_tx: Sender<Option<Vec<identity::PeerAddr>>>,
    pub observed_rx: Receiver<Option<Vec<identity::PeerAddr>>>,
}

#[rustfmt::skip]
impl ReadyChannel {
    #[must_use]
    pub fn subscribe(&self) -> (Receiver<Option<Peer>>, Receiver<Option<Vec<identity::PeerAddr>>>) {
        (self.ready_rx.clone(), self.observed_rx.clone())
    }
}

impl ReadyChannel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ReadyChannel {
    fn default() -> Self {
        let (ready_tx, ready_rx) = watch::channel(None);
        let (observed_tx, observed_rx) = watch::channel(None);
        Self {
            ready_tx,
            ready_rx,
            observed_tx,
            observed_rx,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReadyChannel;
    use crate::identity;

    #[test]
    fn test_usage() {
        let signals = ReadyChannel::default();
        let (ready_rx, observed_rx) = signals.subscribe();
        signals.ready_tx.send_replace(Some(identity::Peer {
            id: identity::PeerId::from("p"),
            addrs: vec![],
        }));
        signals
            .observed_tx
            .send_replace(Some(vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/1")]));
        assert_eq!(ready_rx.borrow().as_ref().map(|r| r.id.as_str()), Some("p"));
        assert!(observed_rx.borrow().is_some());
    }
}
