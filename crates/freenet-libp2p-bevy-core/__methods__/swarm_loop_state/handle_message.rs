use crate::identity;
use crate::p2p;
use libp2p::request_response;

pub fn handle_message<Payload: p2p::traits::Message>(
    state: &mut p2p::SwarmLoopState<Payload>,
    peer: &libp2p::PeerId,
    message: request_response::Message<Payload, Payload>,
) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let payload_clone = request.clone();
            state
                .event_tx
                .send(p2p::Event::Message {
                    from: identity::PeerId(peer.to_string()),
                    payload: request,
                })
                .ok();
            let _ = state
                .swarm
                .behaviour_mut()
                .request_response
                .send_response(channel, payload_clone);
        }
        request_response::Message::Response { response, .. } => {
            state
                .event_tx
                .send(p2p::Event::Message {
                    from: identity::PeerId(peer.to_string()),
                    payload: response,
                })
                .ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use libp2p::request_response;

    use super::handle_message;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let (mut state, mut rx) = p2p::SwarmLoopState::<u32>::for_test();
        let peer = libp2p::PeerId::random();
        let request_id = state
            .swarm
            .behaviour_mut()
            .request_response
            .send_request(&peer, 5);
        handle_message(
            &mut state,
            &peer,
            request_response::Message::Response {
                request_id,
                response: 9,
            },
        );
        assert!(matches!(
            rx.try_recv(),
            Ok(p2p::Event::Message { payload: 9, .. })
        ));
    }
}
