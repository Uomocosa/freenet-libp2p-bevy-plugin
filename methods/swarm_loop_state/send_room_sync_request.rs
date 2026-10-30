use crate::identity;
use crate::p2p;

pub fn send_room_sync_request<Payload: p2p::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    peer_id: &identity::PeerId,
    request: p2p::SnapshotRequest,
) {
    tracing::debug!(target: "p2p", peer = %peer_id, request = ?request, "room sync request sent");
    let Ok(libp2p_id) = peer_id.parse::<libp2p::PeerId>() else {
        state
            .event_tx
            .send(p2p::Event::Net(p2p::NetEvent::RoomSyncRequestFailed {
                from: peer_id.clone(),
                request,
            }))
            .ok();
        return;
    };
    let request_id = state
        .swarm
        .behaviour_mut()
        .room_sync
        .send_request(&libp2p_id, request.clone());
    state
        .room_sync_requests_in_flight
        .insert(request_id, request);
}

#[cfg(test)]
mod tests {
    use super::send_room_sync_request;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = identity::PeerId::from("not-a-peer-id");
        let request = p2p::SnapshotRequest::Fetch {
            room: identity::RoomName::from("r"),
            part: 0,
        };
        send_room_sync_request(&mut state, &peer, request.clone());
        let Ok(p2p::Event::Net(p2p::NetEvent::RoomSyncRequestFailed { from, request: got })) =
            rx.try_recv()
        else {
            panic!("expected a room sync failure");
        };
        assert_eq!((from, got), (peer, request));
        assert!(state.room_sync_requests_in_flight.is_empty());
    }
}
