use crate::identity;
use crate::p2p;
use libp2p::request_response;

pub fn handle_replica_sync<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    peer: &libp2p::PeerId,
    message: request_response::Message<p2p::SnapshotRequest, p2p::SnapshotResponse>,
) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let id = state.snapshot_replies.insert(channel);
            let from = identity::PeerId(peer.to_string());
            match request {
                p2p::SnapshotRequest::Fetch { zone, part } => {
                    state
                        .event_tx
                        .send(p2p::Event::Libp2p(p2p::Libp2pEvent::SnapshotRequest {
                            id,
                            from,
                            zone,
                            part,
                        }))
                        .ok();
                }
                p2p::SnapshotRequest::FetchReplicaSlice { zone, index } => {
                    state
                        .event_tx
                        .send(p2p::Event::Libp2p(p2p::Libp2pEvent::ReplicaSliceRequest {
                            id,
                            from,
                            zone,
                            index,
                        }))
                        .ok();
                }
            }
        }
        request_response::Message::Response {
            request_id,
            response,
            ..
        } => {
            state.replica_sync_requests_in_flight.remove(&request_id);
            tracing::debug!(target: "p2p", peer = %peer, "snapshot response received");
            let from = identity::PeerId(peer.to_string());
            let event = match response {
                p2p::SnapshotResponse::Part {
                    zone,
                    part,
                    total,
                    data,
                } => p2p::Libp2pEvent::SnapshotPart {
                    from,
                    zone,
                    part,
                    total,
                    data,
                },
                p2p::SnapshotResponse::Missing { zone } => {
                    p2p::Libp2pEvent::SnapshotMissing { from, zone }
                }
                p2p::SnapshotResponse::ReplicaSlice { zone, index, data } => {
                    p2p::Libp2pEvent::ReplicaSlice {
                        from,
                        zone,
                        index,
                        data,
                    }
                }
            };
            state.event_tx.send(p2p::Event::Libp2p(event)).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::request_response;

    use super::handle_replica_sync;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = libp2p::PeerId::random();
        let zone = identity::ZoneName::from("zone-1");
        let request_id = state.swarm.behaviour_mut().replica_sync.send_request(
            &peer,
            p2p::SnapshotRequest::Fetch {
                zone: zone.clone(),
                part: 0,
            },
        );
        handle_replica_sync(
            &mut state,
            &peer,
            request_response::Message::Response {
                request_id,
                response: p2p::SnapshotResponse::Missing { zone: zone.clone() },
            },
        );
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Libp2p(p2p::Libp2pEvent::SnapshotMissing { zone: missing, .. }))
                if missing == zone
        ));
    }
}
