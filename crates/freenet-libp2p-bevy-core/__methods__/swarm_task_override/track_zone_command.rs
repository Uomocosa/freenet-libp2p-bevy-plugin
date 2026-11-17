use crate::p2p;

pub fn track_zone_command<Payload: p2p::traits::Message>(
    task: &p2p::SwarmTaskOverride<Payload>,
    command: &p2p::Libp2pCommand,
    subscriptions: &mut p2p::swarm_loop::ZoneSubscriptions,
) {
    let _ = task.orchestrator_port;
    match command {
        p2p::Libp2pCommand::SubscribeZone { zone } => {
            let topic = subscriptions.zone_topic(zone);
            subscriptions.register(&topic, zone.clone());
        }
        p2p::Libp2pCommand::UnsubscribeZone { zone } => {
            let topic = subscriptions.zone_topic(zone);
            subscriptions.unregister(&topic);
        }
        p2p::Libp2pCommand::WatchZone { zone } => {
            let topic = subscriptions.zone_topic(zone);
            subscriptions.watch(&topic, zone.clone());
        }
        p2p::Libp2pCommand::UnwatchZone { zone } => {
            let topic = subscriptions.zone_topic(zone);
            subscriptions.unwatch(&topic, zone);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::track_zone_command;
    use crate::identity;
    use crate::p2p;

    fn task() -> p2p::SwarmTaskOverride<u8> {
        p2p::SwarmTaskOverride::<u8> {
            orchestrator_port: 1,
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            marker: std::marker::PhantomData,
        }
    }

    fn subscriptions() -> p2p::swarm_loop::ZoneSubscriptions {
        p2p::swarm_loop::ZoneSubscriptions::new(identity::GameId::from("game"))
    }

    fn topic(subscriptions: &p2p::swarm_loop::ZoneSubscriptions) -> libp2p::gossipsub::IdentTopic {
        libp2p::gossipsub::IdentTopic::new(
            identity::Topic::from_zone(&subscriptions.game, &identity::ZoneName::from("zone-a"))
                .as_str(),
        )
    }

    #[test]
    fn test_usage() {
        let task = task();
        let mut subscriptions = subscriptions();
        let zone = identity::ZoneName::from("zone-a");
        track_zone_command(
            &task,
            &p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() },
            &mut subscriptions,
        );
        assert_eq!(
            subscriptions.zone(&topic(&subscriptions).hash()),
            Some(&zone)
        );
        assert_eq!(subscriptions.joined, identity::Zone::Restricted(zone));
    }

    #[test]
    fn watching_registers_the_zone_topic_without_joining() {
        let task = task();
        let mut subscriptions = subscriptions();
        let zone = identity::ZoneName::from("zone-a");
        let command = p2p::Libp2pCommand::WatchZone { zone: zone.clone() };
        track_zone_command(&task, &command, &mut subscriptions);
        assert_eq!(
            subscriptions.zone(&topic(&subscriptions).hash()),
            Some(&zone)
        );
        assert_eq!(subscriptions.joined, identity::Zone::Global);
        let command = p2p::Libp2pCommand::UnwatchZone { zone };
        track_zone_command(&task, &command, &mut subscriptions);
        assert_eq!(subscriptions.zone(&topic(&subscriptions).hash()), None);
    }

    #[test]
    fn a_second_watch_keeps_the_zone_topic_after_one_unwatch() {
        let task = task();
        let mut subscriptions = subscriptions();
        let zone = identity::ZoneName::from("zone-a");
        for _ in 0..2 {
            track_zone_command(
                &task,
                &p2p::Libp2pCommand::WatchZone { zone: zone.clone() },
                &mut subscriptions,
            );
        }
        track_zone_command(
            &task,
            &p2p::Libp2pCommand::UnwatchZone { zone: zone.clone() },
            &mut subscriptions,
        );
        assert_eq!(
            subscriptions.zone(&topic(&subscriptions).hash()),
            Some(&zone)
        );
        track_zone_command(
            &task,
            &p2p::Libp2pCommand::UnwatchZone { zone },
            &mut subscriptions,
        );
        assert_eq!(subscriptions.zone(&topic(&subscriptions).hash()), None);
    }

    #[test]
    fn unwatching_an_unknown_zone_changes_nothing() {
        let task = task();
        let mut subscriptions = subscriptions();
        let zone = identity::ZoneName::from("zone-a");
        track_zone_command(
            &task,
            &p2p::Libp2pCommand::UnwatchZone { zone },
            &mut subscriptions,
        );
        assert_eq!(subscriptions.zone(&topic(&subscriptions).hash()), None);
        assert!(subscriptions.watched.is_empty());
    }

    #[test]
    fn unwatching_a_joined_zone_keeps_the_zone_topic() {
        let task = task();
        let mut subscriptions = subscriptions();
        let zone = identity::ZoneName::from("zone-a");
        let join = p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() };
        track_zone_command(&task, &join, &mut subscriptions);
        let watch = p2p::Libp2pCommand::WatchZone { zone: zone.clone() };
        track_zone_command(&task, &watch, &mut subscriptions);
        let unwatch = p2p::Libp2pCommand::UnwatchZone { zone: zone.clone() };
        track_zone_command(&task, &unwatch, &mut subscriptions);
        assert_eq!(
            subscriptions.zone(&topic(&subscriptions).hash()),
            Some(&zone)
        );
        assert_eq!(subscriptions.joined, identity::Zone::Restricted(zone));
    }

    #[test]
    fn leaving_a_watched_zone_keeps_the_zone_topic() {
        let task = task();
        let mut subscriptions = subscriptions();
        let zone = identity::ZoneName::from("zone-a");
        let watch = p2p::Libp2pCommand::WatchZone { zone: zone.clone() };
        track_zone_command(&task, &watch, &mut subscriptions);
        let join = p2p::Libp2pCommand::SubscribeZone { zone: zone.clone() };
        track_zone_command(&task, &join, &mut subscriptions);
        let leave = p2p::Libp2pCommand::UnsubscribeZone { zone: zone.clone() };
        track_zone_command(&task, &leave, &mut subscriptions);
        assert_eq!(
            subscriptions.zone(&topic(&subscriptions).hash()),
            Some(&zone)
        );
        assert_eq!(subscriptions.joined, identity::Zone::Global);
        let unwatch = p2p::Libp2pCommand::UnwatchZone { zone };
        track_zone_command(&task, &unwatch, &mut subscriptions);
        assert_eq!(subscriptions.zone(&topic(&subscriptions).hash()), None);
    }
}
