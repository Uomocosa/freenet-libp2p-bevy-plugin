use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;

pub fn track_zone_command<Payload: p2p::traits::Message>(
    task: &p2p::SwarmTaskOverride<Payload>,
    command: &p2p::Libp2pCommand,
    game: &identity::GameId,
    zones_by_topic: &mut BTreeMap<identity::Topic, identity::ZoneName>,
) {
    let _ = task.orchestrator_port;
    match command {
        p2p::Libp2pCommand::SubscribeZone { zone } => {
            zones_by_topic.insert(identity::Topic::from_zone(game, zone), zone.clone());
        }
        p2p::Libp2pCommand::UnsubscribeZone { zone } => {
            zones_by_topic.remove(&identity::Topic::from_zone(game, zone));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::track_zone_command;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let task = p2p::SwarmTaskOverride::<u8> {
            orchestrator_port: 1,
            peer: identity::Peer {
                id: identity::PeerId::from("peer"),
                addrs: Vec::new(),
            },
            marker: std::marker::PhantomData,
        };
        let mut zones = BTreeMap::new();
        let game = identity::GameId::from("game");
        track_zone_command(
            &task,
            &p2p::Libp2pCommand::SubscribeZone {
                zone: identity::ZoneName::from("zone-a"),
            },
            &game,
            &mut zones,
        );
        assert_eq!(zones.len(), 1);
    }
}
