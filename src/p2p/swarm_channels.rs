use atomic_delegate_macros::atomic_delegates;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver as WatchReceiver;

use crate::identity;
use crate::p2p;

pub struct SwarmChannels {
    pub commands: UnboundedSender<p2p::Libp2pCommand>,
    pub events: Receiver<p2p::Libp2pEvent>,
    pub ready: WatchReceiver<Option<identity::Peer>>,
    pub observed: WatchReceiver<Option<Vec<identity::PeerAddr>>>,
}

#[atomic_delegates]
impl SwarmChannels {
    pub async fn wait_ready(&mut self) -> Option<identity::Peer> {}
}

#[cfg(test)]
mod tests {
    use super::SwarmChannels;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (_ready_tx, ready_rx) = tokio::sync::watch::channel::<Option<identity::Peer>>(None);
        let (_observed_tx, observed_rx) =
            tokio::sync::watch::channel::<Option<Vec<identity::PeerAddr>>>(None);
        let (commands_tx, _commands_rx) =
            tokio::sync::mpsc::unbounded_channel::<p2p::Libp2pCommand>();
        let (_events_tx, events_rx) = tokio::sync::broadcast::channel::<p2p::Libp2pEvent>(16);
        let channels = SwarmChannels {
            commands: commands_tx,
            events: events_rx,
            ready: ready_rx,
            observed: observed_rx,
        };
        assert!(channels.ready.borrow().is_none());
        let _ = identity::PeerId::from("me");
    }
}
