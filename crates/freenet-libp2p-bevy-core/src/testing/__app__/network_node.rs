use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn network_node<Payload: p2p::traits::Message>(
    game: &identity::GameId,
) -> testing::Node<Payload> {
    let (commands, command_rx) = tokio::sync::mpsc::unbounded_channel();
    let (libp2p_commands, libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
    let (event_tx, events) = tokio::sync::mpsc::unbounded_channel();
    let config = p2p::RunnerConfig {
        mode: p2p::TransportMode::Tcp,
        mdns: p2p::MdnsMode::Disabled,
        game: game.clone(),
        gossipsub: p2p::GossipsubConfig::default(),
    };
    let handle = config.spawn_runner::<Payload>(
        command_rx,
        libp2p_rx,
        event_tx,
        libp2p::identity::Keypair::generate_ed25519(),
    );
    testing::Node {
        commands,
        libp2p_commands,
        events,
        handle,
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::network_node;

    #[tokio::test]
    async fn test_usage() {
        let node = network_node::<testing::DummyMessage>(&identity::GameId::from("game"));
        assert!(!node.handle.is_finished());
        node.handle.abort();
    }
}
