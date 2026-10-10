use std::sync::Mutex;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::net_id;
use crate::p2p;

pub struct Config<T: p2p::Message> {
    pub own_id: net_id::NetworkId,
    pub cmd_tx: UnboundedSender<p2p::Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<p2p::Event<T>>>>,
}

#[rustfmt::skip]
impl<T: p2p::Message> Config<T> {
    pub fn take_event_rx(&self) -> Option<UnboundedReceiver<p2p::Event<T>>> {
        self.event_rx.lock().ok()?.take()
    }
}

impl<T: p2p::Message> Config<T> {
    #[must_use]
    pub const fn new(
        own_id: net_id::NetworkId,
        cmd_tx: UnboundedSender<p2p::Command<T>>,
        event_rx: UnboundedReceiver<p2p::Event<T>>,
    ) -> Self {
        Self {
            own_id,
            cmd_tx,
            event_rx: Mutex::new(Some(event_rx)),
        }
    }
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;

    use super::Config;
    use crate::net_id;
    use crate::p2p;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Deref)]
    struct Dummy(u32);

    #[test]
    fn test_usage() {
        let (cmd_tx, _) = mpsc::unbounded_channel();
        let (_, event_rx) = mpsc::unbounded_channel::<p2p::Event<Dummy>>();
        let cfg = Config::new(net_id::NetworkId(1), cmd_tx, event_rx);
        assert!(cfg.take_event_rx().is_some());
        assert!(cfg.take_event_rx().is_none());
    }
}
