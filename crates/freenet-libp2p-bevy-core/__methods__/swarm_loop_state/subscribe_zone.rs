use crate::identity;
use crate::p2p;
use libp2p::gossipsub;

pub fn subscribe_zone<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    zone: identity::ZoneName,
) {
    let topic = gossipsub::IdentTopic::new(
        identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
    );
    match state.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
        Ok(_) => {
            tracing::info!(target: "p2p", zone = %zone.as_str(), "p2p zone subscribed");
            state.subscriptions.register(&topic, zone);
        }
        Err(error) => {
            state
                .event_tx
                .send(p2p::Event::Error(format!(
                    "zone subscribe {}: {error}",
                    zone.as_str()
                )))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::subscribe_zone;

    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        subscribe_zone(&mut state, identity::ZoneName::from("zone-a"));
        assert!(state.subscriptions.joined.restricted().is_some());
    }
}
