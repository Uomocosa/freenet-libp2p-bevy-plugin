use std::time::Duration;

use crate::identity;
use crate::p2p;

const READY_TIMEOUT: Duration = Duration::from_secs(120);

pub async fn wait_ready(channels: &mut p2p::SwarmChannels) -> Option<identity::Peer> {
    let wait = channels.ready.wait_for(Option::is_some);
    match tokio::time::timeout(READY_TIMEOUT, wait).await {
        Ok(Ok(value)) => value.clone(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::wait_ready;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let ready = identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        };
        let (_tx, ready_rx) = tokio::sync::watch::channel(Some(ready.clone()));
        let (_observed_tx, observed_rx) =
            tokio::sync::watch::channel::<Option<Vec<identity::PeerAddr>>>(None);
        let (libp2p_commands_tx, _libp2p_rx) =
            tokio::sync::mpsc::unbounded_channel::<p2p::Libp2pCommand>();
        let (_events_tx, events_rx) = tokio::sync::broadcast::channel::<p2p::Libp2pEvent>(16);
        let mut channels = p2p::SwarmChannels {
            commands: libp2p_commands_tx,
            events: events_rx,
            ready: ready_rx,
            observed: observed_rx,
        };
        assert_eq!(wait_ready(&mut channels).await, Some(ready));
    }
}
