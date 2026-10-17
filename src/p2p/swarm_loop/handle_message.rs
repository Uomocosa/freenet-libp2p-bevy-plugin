use libp2p::request_response;

use crate::identity;
use crate::p2p;

pub fn handle_message<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    peer: &libp2p::PeerId,
    message: request_response::Message<Payload, Payload>,
) {
    match message {
        request_response::Message::Request {
            request, channel, ..
        } => {
            let payload_clone = request.clone();
            event_tx
                .send(p2p::Event::Message {
                    from: identity::PeerId(peer.to_string()),
                    payload: request,
                })
                .ok();
            let _ = swarm
                .behaviour_mut()
                .request_response
                .send_response(channel, payload_clone);
        }
        request_response::Message::Response { response, .. } => {
            event_tx
                .send(p2p::Event::Message {
                    from: identity::PeerId(peer.to_string()),
                    payload: response,
                })
                .ok();
        }
    }
}

// no test_usage necessary
