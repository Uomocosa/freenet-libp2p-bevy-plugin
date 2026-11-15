use crate::identity;
use crate::p2p;
use libp2p::gossipsub;

pub fn unsubscribe_zone<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    zone: &identity::ZoneName,
) {
    let topic = gossipsub::IdentTopic::new(
        identity::Topic::from_zone(&state.subscriptions.game, zone).as_str(),
    );
    let _ = state.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
    state.subscriptions.unregister(&topic);
    tracing::info!(target: "p2p", zone = %zone.as_str(), "p2p zone unsubscribed");
}

#[cfg(test)]
mod tests {
    use super::unsubscribe_zone;

    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = p2p::SwarmLoopState::<u32>::for_test();
        let zone = identity::ZoneName::from("zone-a");
        state.subscribe_zone(zone.clone());
        unsubscribe_zone(&mut state, &zone);
        assert!(state.subscriptions.joined.restricted().is_none());
    }
}
