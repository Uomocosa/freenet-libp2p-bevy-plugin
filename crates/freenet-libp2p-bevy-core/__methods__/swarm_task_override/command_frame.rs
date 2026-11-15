use std::collections::BTreeMap;

use crate::identity;
use crate::p2p;
use crate::testing;

pub fn command_frame<Payload: p2p::traits::Message>(
    task: &p2p::SwarmTaskOverride<Payload>,
    game: &identity::GameId,
    zones_by_topic: &mut BTreeMap<identity::Topic, identity::ZoneName>,
    events: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    cmd: p2p::Command<Payload>,
) -> Option<testing::FakeWireClientFrame> {
    let _ = task.orchestrator_port;
    match cmd {
        p2p::Command::Net(command) => {
            task.track_zone_command(&command, game, zones_by_topic);
            Some(testing::FakeWireClientFrame::Libp2pCommand(command))
        }
        p2p::Command::PublishInJoinedZone { payload } => {
            let Ok(data) = bincode::serialize(&payload) else {
                events
                    .send(p2p::Event::Error(
                        "fake network: zone payload encode failed".to_string(),
                    ))
                    .ok();
                return None;
            };
            Some(testing::FakeWireClientFrame::JoinedZoneBytes(data))
        }
        p2p::Command::PublishToSpecificZone { zone, payload } => {
            let Ok(data) = bincode::serialize(&payload) else {
                events
                    .send(p2p::Event::Error(
                        "fake network: zone payload encode failed".to_string(),
                    ))
                    .ok();
                return None;
            };
            Some(testing::FakeWireClientFrame::SpecificZoneBytes { zone, data })
        }
        p2p::Command::Send { peer_id, .. } => {
            tracing::warn!(
                target: "p2p",
                peer = %peer_id.as_str(),
                "fake network: Command::Send has no callers, dropping"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::command_frame;
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
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();
        let frame = command_frame(
            &task,
            &identity::GameId::from("game"),
            &mut BTreeMap::new(),
            &event_tx,
            p2p::Command::PublishInJoinedZone { payload: 7u8 },
        );
        assert!(matches!(
            frame,
            Some(crate::testing::FakeWireClientFrame::JoinedZoneBytes(_))
        ));
    }
}
