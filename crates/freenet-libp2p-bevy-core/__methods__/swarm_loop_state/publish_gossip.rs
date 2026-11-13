use libp2p::gossipsub;

use crate::identity;
use crate::p2p;

pub fn publish_gossip<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    topic: &identity::Topic,
    data: Vec<u8>,
) {
    let topic = gossipsub::IdentTopic::new(topic.as_str());
    let subscribed_peers = state
        .swarm
        .behaviour()
        .gossipsub
        .all_peers()
        .filter(|(_, topics)| topics.iter().any(|subscribed| **subscribed == topic.hash()))
        .count();
    tracing::debug!(target: "p2p", topic = %topic, subscribed_peers, "p2p gossip publish");
    if let Err(error) = state.swarm.behaviour_mut().gossipsub.publish(topic, data) {
        state
            .event_tx
            .send(p2p::Event::Error(format!("gossip publish: {error}")))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::publish_gossip;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        publish_gossip(&mut state, &identity::Topic::from("lobby/topic"), vec![1]);
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Error(message)) if message.starts_with("gossip publish")
        ));
    }
}
