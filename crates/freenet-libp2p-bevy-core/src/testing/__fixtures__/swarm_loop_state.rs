use libp2p::identity::Keypair;

use crate::identity;
use crate::p2p;

pub fn swarm_loop_state<Payload: p2p::traits::Message>() -> Result<
    (
        p2p::SwarmLoopState<Payload>,
        tokio::sync::mpsc::UnboundedReceiver<p2p::Event<Payload>>,
    ),
    String,
> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let config = p2p::RunnerConfig {
        mode: p2p::TransportMode::Both,
        mdns: p2p::MdnsMode::Disabled,
        game: identity::GameId::from("test"),
        gossipsub: p2p::GossipsubConfig::default(),
    };
    let swarm = p2p::RunnerConfig::build_swarm::<Payload>(&config, Keypair::generate_ed25519())?;
    let state = p2p::SwarmLoopState::new(
        swarm,
        tx,
        p2p::swarm_loop::ZoneSubscriptions::new(config.game.clone()),
        p2p::swarm_loop::PendingSnapshotReplies::default(),
        config.mode,
    );
    Ok((state, rx))
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;

    use super::swarm_loop_state;

    #[test]
    fn test_usage() {
        let (state, _rx) = swarm_loop_state::<u32>().expect("swarm loop state");
        assert_eq!(state.subscriptions.game, identity::GameId::from("test"));
        assert_eq!(state.mode, p2p::TransportMode::Both);
    }
}
