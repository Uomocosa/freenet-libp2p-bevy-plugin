use crate::identity;
use crate::p2p;
use libp2p::gossipsub;

pub fn watch_zone<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    zone: identity::ZoneName,
) {
    let topic = gossipsub::IdentTopic::new(
        identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
    );
    match state.swarm.behaviour_mut().gossipsub.subscribe(&topic) {
        Ok(_) => {
            tracing::info!(target: "p2p", zone = %zone.as_str(), "p2p zone watched");
            state.subscriptions.watch(&topic, zone);
        }
        Err(error) => {
            state
                .event_tx
                .send(p2p::Event::Error(format!(
                    "zone watch {}: {error}",
                    zone.as_str()
                )))
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::watch_zone;

    use crate::identity;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let zone = identity::ZoneName::from("zone-a");
        watch_zone(&mut state, zone.clone());
        let topic = libp2p::gossipsub::IdentTopic::new(
            identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
        );
        assert_eq!(state.subscriptions.zone(&topic.hash()), Some(&zone));
        assert!(state.subscriptions.joined.restricted().is_none());
    }
}
