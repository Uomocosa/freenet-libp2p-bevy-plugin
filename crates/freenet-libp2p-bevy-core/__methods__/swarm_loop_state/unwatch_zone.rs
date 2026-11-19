use crate::identity;
use crate::p2p;
use libp2p::gossipsub;

pub fn unwatch_zone<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    zone: &identity::ZoneName,
) {
    let topic = gossipsub::IdentTopic::new(
        identity::Topic::from_zone(&state.subscriptions.game, zone).as_str(),
    );
    let did_last_watcher_leave = state.subscriptions.unwatch(&topic, zone);
    if did_last_watcher_leave && state.subscriptions.zone(&topic.hash()).is_none() {
        let _ = state.swarm.behaviour_mut().gossipsub.unsubscribe(&topic);
    }
    tracing::info!(target: "p2p", zone = %zone.as_str(), "p2p zone unwatched");
}

#[cfg(test)]
mod tests {
    use super::unwatch_zone;

    use crate::identity;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, _rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let zone = identity::ZoneName::from("zone-a");
        state.watch_zone(zone.clone());
        unwatch_zone(&mut state, &zone);
        let topic = libp2p::gossipsub::IdentTopic::new(
            identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
        );
        assert_eq!(state.subscriptions.zone(&topic.hash()), None);
    }

    #[tokio::test]
    async fn a_second_watcher_keeps_the_zone_topic() {
        let (mut state, _rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let zone = identity::ZoneName::from("zone-a");
        let topic = libp2p::gossipsub::IdentTopic::new(
            identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
        );
        state.watch_zone(zone.clone());
        state.watch_zone(zone.clone());
        unwatch_zone(&mut state, &zone);
        assert_eq!(state.subscriptions.zone(&topic.hash()), Some(&zone));
        unwatch_zone(&mut state, &zone);
        assert_eq!(state.subscriptions.zone(&topic.hash()), None);
    }

    #[tokio::test]
    async fn unwatching_an_unknown_zone_keeps_the_topic_gone() {
        let (mut state, _rx) = testing::swarm_loop_state::<u32>().expect("swarm loop state");
        let zone = identity::ZoneName::from("zone-a");
        let topic = libp2p::gossipsub::IdentTopic::new(
            identity::Topic::from_zone(&state.subscriptions.game, &zone).as_str(),
        );
        unwatch_zone(&mut state, &zone);
        assert_eq!(state.subscriptions.zone(&topic.hash()), None);
        assert!(state.subscriptions.watched.is_empty());
    }
}
