use crate::identity;
use crate::p2p;
use libp2p::request_response;

pub fn handle_room_sync<Payload: p2p::traits::Message>(
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
                p2p::SnapshotRequest::Fetch { room, part } => {
                    state
                        .event_tx
                        .send(p2p::Event::Libp2p(p2p::Libp2pEvent::SnapshotRequest {
                            id,
                            from,
                            room,
                            part,
                        }))
                        .ok();
                }
                p2p::SnapshotRequest::FetchReplicaSlice { room, index } => {
                    state
                        .event_tx
                        .send(p2p::Event::Libp2p(p2p::Libp2pEvent::ReplicaSliceRequest {
                            id,
                            from,
                            room,
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
            state.room_sync_requests_in_flight.remove(&request_id);
            tracing::debug!(target: "p2p", peer = %peer, "snapshot response received");
            let from = identity::PeerId(peer.to_string());
            let event = match response {
                p2p::SnapshotResponse::Part {
                    room,
                    part,
                    total,
                    data,
                } => p2p::Libp2pEvent::SnapshotPart {
                    from,
                    room,
                    part,
                    total,
                    data,
                },
                p2p::SnapshotResponse::Missing { room } => {
                    p2p::Libp2pEvent::SnapshotMissing { from, room }
                }
                p2p::SnapshotResponse::ReplicaSlice { room, index, data } => {
                    p2p::Libp2pEvent::ReplicaSlice {
                        from,
                        room,
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

    use super::handle_room_sync;
    use crate::identity;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = libp2p::PeerId::random();
        let room = identity::RoomName::from("room-1");
        let request_id = state.swarm.behaviour_mut().room_sync.send_request(
            &peer,
            p2p::SnapshotRequest::Fetch {
                room: room.clone(),
                part: 0,
            },
        );
        handle_room_sync(
            &mut state,
            &peer,
            request_response::Message::Response {
                request_id,
                response: p2p::SnapshotResponse::Missing { room: room.clone() },
            },
        );
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Libp2p(p2p::Libp2pEvent::SnapshotMissing { room: missing, .. }))
                if missing == room
        ));
    }
}
