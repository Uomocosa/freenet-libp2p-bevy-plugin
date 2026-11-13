use crate::identity;
use crate::p2p;
use libp2p::request_response;

pub fn handle_peer_exchange<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    peer: &libp2p::PeerId,
    message: request_response::Message<Vec<u8>, Vec<u8>>,
) {
    if let request_response::Message::Request {
        request, channel, ..
    } = message
    {
        state
            .event_tx
            .send(p2p::Event::Libp2p(p2p::Libp2pEvent::PeerExchange {
                from: identity::PeerId(peer.to_string()),
                data: request,
            }))
            .ok();
        let _ = state
            .swarm
            .behaviour_mut()
            .peer_exchange
            .send_response(channel, Vec::new());
    }
}

#[cfg(test)]
mod tests {
    use libp2p::request_response;

    use super::handle_peer_exchange;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = libp2p::PeerId::random();
        let request_id = state
            .swarm
            .behaviour_mut()
            .peer_exchange
            .send_request(&peer, vec![1]);
        handle_peer_exchange(
            &mut state,
            &peer,
            request_response::Message::Response {
                request_id,
                response: vec![2],
            },
        );
        assert!(rx.try_recv().is_err());
    }
}
