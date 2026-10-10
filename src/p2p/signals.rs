use bevy::prelude::Resource;
use tokio::sync::watch::{self, Receiver, Sender};

use crate::net_id;
use net_id::Peer;

#[derive(Resource)]
pub struct Signals {
    pub ready_tx: Sender<Option<Peer>>,
    pub ready_rx: Receiver<Option<Peer>>,
    pub observed_tx: Sender<Option<Vec<net_id::PeerAddr>>>,
    pub observed_rx: Receiver<Option<Vec<net_id::PeerAddr>>>,
}

#[rustfmt::skip]
impl Signals {
    #[must_use]
    pub fn subscribe(&self) -> (Receiver<Option<Peer>>, Receiver<Option<Vec<net_id::PeerAddr>>>) {
        (self.ready_rx.clone(), self.observed_rx.clone())
    }
}

impl Signals {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Signals {
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
    use super::Signals;
    use crate::net_id;

    #[test]
    fn test_usage() {
        let signals = Signals::default();
        let (ready_rx, observed_rx) = signals.subscribe();
        signals.ready_tx.send_replace(Some(net_id::Peer {
            id: net_id::PeerId::from("p"),
            addrs: vec![],
        }));
        signals.observed_tx.send_replace(Some(vec![net_id::PeerAddr(
            "/ip4/1.2.3.4/tcp/1".to_string(),
        )]));
        assert_eq!(ready_rx.borrow().as_ref().map(|r| r.id.as_str()), Some("p"));
        assert!(observed_rx.borrow().is_some());
    }
}
