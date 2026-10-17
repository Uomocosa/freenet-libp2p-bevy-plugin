use libp2p::request_response;

use crate::identity;
use crate::p2p;

pub fn handle_room_sync<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
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
            };
            event_tx.send(p2p::Event::Net(event)).ok();
        }
    }
}

// no test_usage necessary
