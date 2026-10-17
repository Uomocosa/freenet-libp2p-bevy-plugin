use libp2p::request_response;

use crate::identity;
use crate::p2p;

pub fn handle_room_sync<Payload: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    snapshot_replies: &mut p2p::swarm_loop::PendingSnapshotReplies,
    peer: &libp2p::PeerId,
    message: request_response::Message<p2p::SnapshotRequest, p2p::SnapshotResponse>,
) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let id = snapshot_replies.insert(channel);
            let from = identity::PeerId(peer.to_string());
            match request {
                p2p::SnapshotRequest::Fetch { room, part } => {
                    event_tx
                        .send(p2p::Event::Net(p2p::NetEvent::SnapshotRequest {
                            id,
                            from,
                            room,
                            part,
                        }))
                        .ok();
                }
                p2p::SnapshotRequest::FetchReplicaSlice { room, index } => {
                    event_tx
                        .send(p2p::Event::Net(p2p::NetEvent::ReplicaSliceRequest {
                            id,
                            from,
                            room,
                            index,
                        }))
                        .ok();
                }
            }
        }
        request_response::Message::Response { response, .. } => {
            let from = identity::PeerId(peer.to_string());
            let event = match response {
                p2p::SnapshotResponse::Part {
                    room,
                    part,
                    total,
                    data,
                } => p2p::NetEvent::SnapshotPart {
                    from,
                    room,
                    part,
                    total,
                    data,
                },
                p2p::SnapshotResponse::Missing { room } => {
                    p2p::NetEvent::SnapshotMissing { from, room }
                }
                p2p::SnapshotResponse::ReplicaSlice { room, index, data } => {
                    p2p::NetEvent::ReplicaSlice {
                        from,
                        room,
                        index,
                        data,
                    }
                }
            };
            event_tx.send(p2p::Event::Net(event)).ok();
        }
    }
}

// no test_usage necessary
