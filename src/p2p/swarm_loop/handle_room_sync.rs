use libp2p::request_response;

use crate::identity;
use crate::p2p;

pub fn handle_room_sync<T: p2p::Message>(
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    rooms: &mut p2p::swarm_loop::RoomLoop,
    peer: &libp2p::PeerId,
    message: request_response::Message<p2p::RoomSyncFrame, p2p::RoomSyncFrame>,
) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let id = rooms.next_request;
            rooms.next_request = rooms.next_request.saturating_add(1);
            rooms.pending.insert(id, channel);
            let from = identity::PeerId(peer.to_string());
            match request {
                p2p::RoomSyncFrame::Fetch { room, part } => {
                    event_tx
                        .send(p2p::Event::Net(p2p::NetEvent::SnapshotRequest {
                            id,
                            from,
                            room,
                            part,
                        }))
                        .ok();
                }
                other => {
                    event_tx
                        .send(p2p::Event::Error(format!(
                            "room sync: unexpected request {other:?}"
                        )))
                        .ok();
                }
            }
        }
        request_response::Message::Response { response, .. } => {
            let from = identity::PeerId(peer.to_string());
            let event = match response {
                p2p::RoomSyncFrame::Part {
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
                p2p::RoomSyncFrame::Missing { room } => {
                    p2p::NetEvent::SnapshotMissing { from, room }
                }
                p2p::RoomSyncFrame::Fetch { .. } => {
                    event_tx
                        .send(p2p::Event::Error(
                            "room sync: peer answered a fetch with a fetch".to_string(),
                        ))
                        .ok();
                    return;
                }
            };
            event_tx.send(p2p::Event::Net(event)).ok();
        }
    }
}

// no test_usage necessary
