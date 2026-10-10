use libp2p::request_response;

use crate::net_id;
use crate::p2p;

pub fn handle_exchange<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    peer: &libp2p::PeerId,
    message: request_response::Message<Vec<u8>, Vec<u8>>,
) {
    if let request_response::Message::Request {
        request, channel, ..
    } = message
    {
        event_tx
            .send(p2p::Event::Net(p2p::NetEvent::Exchange {
                from: net_id::PeerId(peer.to_string()),
                data: request,
            }))
            .ok();
        let _ = swarm
            .behaviour_mut()
            .exchange
            .send_response(channel, Vec::new());
    }
}

// no test_usage necessary
