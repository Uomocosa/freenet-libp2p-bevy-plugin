use crate::net_id;
use crate::p2p;

pub fn dispatch_command<T: p2p::Message>(
    swarm: &mut libp2p::Swarm<p2p::Behaviour<T>>,
    event_tx: &tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    room_queries: &mut std::collections::HashMap<libp2p::kad::QueryId, net_id::RoomName>,
    cmd: Option<p2p::Command<T>>,
) -> bool {
    match cmd {
        Some(p2p::Command::Net(net)) => {
            p2p::swarm_loop::dispatch_net_command(swarm, event_tx, room_queries, net);
        }
        Some(p2p::Command::Send { peer_id, payload }) => {
            if let Ok(pid) = peer_id.parse::<libp2p::PeerId>() {
                swarm
                    .behaviour_mut()
                    .request_response
                    .send_request(&pid, payload);
            }
        }
        None => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use libp2p::identity::Keypair;

    use super::dispatch_command;
    use crate::net_id;
    use crate::p2p;

    #[tokio::test]
    async fn test_usage() {
        let mut swarm =
            p2p::build_swarm::<u32>(Keypair::generate_ed25519(), p2p::MdnsMode::Disabled).unwrap();
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let mut room_queries = HashMap::new();
        let subscribe = p2p::Command::Net(p2p::NetCommand::Subscribe {
            topic: net_id::Topic::from("lobby/topic"),
        });
        assert!(dispatch_command(
            &mut swarm,
            &tx,
            &mut room_queries,
            Some(subscribe)
        ));
        assert_eq!(swarm.behaviour().gossipsub.topics().count(), 1);
        assert!(!dispatch_command(&mut swarm, &tx, &mut room_queries, None));
    }
}
