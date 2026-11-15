use crate::identity;
use crate::p2p;

pub fn publish_zone<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    config: &p2p::GossipsubConfig,
    zone: &identity::ZoneName,
    payload: &Payload,
) {
    let Ok(data) = bincode::serialize(payload) else {
        state
            .event_tx
            .send(p2p::Event::Error(format!(
                "zone publish {}: encode failed",
                zone.as_str()
            )))
            .ok();
        return;
    };
    if data.len() > config.max_transmit_size {
        state
            .event_tx
            .send(p2p::Event::Error(format!(
                "zone publish {}: payload {} bytes exceeds limit {}",
                zone.as_str(),
                data.len(),
                config.max_transmit_size
            )))
            .ok();
        return;
    }
    let topic = libp2p::gossipsub::IdentTopic::new(
        identity::Topic::from_zone(&state.subscriptions.game, zone).as_str(),
    );
    if let Err(error) = state.swarm.behaviour_mut().gossipsub.publish(topic, data) {
        state
            .event_tx
            .send(p2p::Event::Error(format!(
                "zone publish {}: {error}",
                zone.as_str()
            )))
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::publish_zone;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<Vec<u8>>::for_test();
        let config = p2p::GossipsubConfig::default();
        let zone = identity::ZoneName::from("zone-a");
        publish_zone(&mut state, &config, &zone, &vec![0; 70_000]);
        let Ok(p2p::Event::Error(message)) = rx.try_recv() else {
            panic!("expected an error event");
        };
        assert!(message.contains("exceeds limit"));
    }
}
