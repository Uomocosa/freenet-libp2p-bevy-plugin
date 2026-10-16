use std::sync::Mutex;

use bevy::prelude::Resource;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::p2p;
use p2p::NetEvent;

#[derive(Resource)]
pub struct NetEventChannel {
    pub tx: UnboundedSender<NetEvent>,
    pub rx: Mutex<Option<UnboundedReceiver<NetEvent>>>,
}

#[rustfmt::skip]
impl NetEventChannel {
    pub fn take_rx(&self) -> Option<UnboundedReceiver<NetEvent>> {
        self.rx.lock().ok()?.take()
    }
}

impl NetEventChannel {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for NetEventChannel {
    fn default() -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            tx,
            rx: Mutex::new(Some(rx)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NetEventChannel;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let tap = NetEventChannel::default();
        assert!(
            tap.tx
                .send(p2p::NetEvent::PeerConnected(identity::PeerId::from("p")))
                .is_ok()
        );
        assert!(tap.take_rx().is_some());
        assert!(tap.take_rx().is_none());
    }
}
