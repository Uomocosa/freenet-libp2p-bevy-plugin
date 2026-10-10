use libp2p::identity::Keypair;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::p2p;

#[must_use]
pub fn spawn_runner<T: p2p::Message>(
    cmd_rx: UnboundedReceiver<p2p::Command<T>>,
    net_rx: UnboundedReceiver<p2p::NetCommand>,
    event_tx: UnboundedSender<p2p::Event<T>>,
    keypair: Keypair,
    mode: p2p::TransportMode,
    mdns: p2p::MdnsMode,
) -> JoinHandle<()> {
    tokio::spawn(p2p::run(cmd_rx, net_rx, event_tx, keypair, mode, mdns))
}

#[cfg(test)]
mod tests {
    use super::spawn_runner;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (_cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_net_tx, net_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let handle = spawn_runner::<()>(
            cmd_rx,
            net_rx,
            event_tx,
            keypair,
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        );
        let found = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            while let Some(event) = event_rx.recv().await {
                if matches!(event, p2p::Event::Net(p2p::NetEvent::Ready(_))) {
                    break;
                }
            }
        })
        .await;
        handle.abort();
        assert!(found.is_ok());
    }
}
