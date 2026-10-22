use atomic_delegate_macros::atomic_delegates;

use crate::identity;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerConfig {
    pub mode: p2p::TransportMode,
    pub mdns: p2p::MdnsMode,
    pub game: identity::GameId,
    pub gossipsub: p2p::GossipsubConfig,
}

#[atomic_delegates]
impl RunnerConfig {
    pub fn build_swarm<Payload: p2p::Message>(
        &self,
        keypair: libp2p::identity::Keypair,
    ) -> Result<libp2p::Swarm<p2p::Behaviour<Payload>>, String> {
    }
    pub async fn run<Payload: p2p::Message>(
        &self,
        cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
        net_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::NetCommand>,
        event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
        keypair: libp2p::identity::Keypair,
    ) {
    }
    pub fn spawn_runner<Payload: p2p::Message>(
        &self,
        cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
        net_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::NetCommand>,
        event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
        keypair: libp2p::identity::Keypair,
    ) -> tokio::task::JoinHandle<()> {
    }
}

#[cfg(test)]
mod tests {
    use super::RunnerConfig;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let config = RunnerConfig {
            mode: p2p::TransportMode::Both,
            mdns: p2p::MdnsMode::Disabled,
            game: identity::GameId::from("test"),
            gossipsub: p2p::GossipsubConfig::default(),
        };
        assert_eq!(config.game.as_str(), "test");
    }
}
