use libp2p::request_response;

use crate::identity;
use crate::p2p;

pub fn handle_peer_exchange<Payload: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<Payload>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    peer: &libp2p::PeerId,
    message: request_response::Message<Vec<u8>, Vec<u8>>,
) {
    if let request_response::Message::Request {
        request, channel, ..
    } = message
    {
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::PeerExchange {
                from: identity::PeerId(peer.to_string()),
                data: request,
            }))
            .ok();
        let _ = swarm
            .behaviour_mut()
            .peer_exchange
            .send_response(channel, Vec::new());
    }
}

// no test_usage necessary
