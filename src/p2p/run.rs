use std::time::Duration;

use futures::StreamExt;
use libp2p::identity::Keypair;

use crate::net_id;
use crate::p2p;

pub async fn run<T: p2p::Message>(
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<T>>,
    mut net_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::NetCommand>,
    event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<T>>,
    keypair: Keypair,
    mode: p2p::TransportMode,
    mdns: p2p::MdnsMode,
) {
    let mut swarm = match p2p::build_swarm::build_swarm::<T>(keypair, mdns) {
        Ok(s) => s,
        Err(e) => {
            event_tx.send(p2p::Event::Error(e)).ok();
            return;
        }
    };

    if mode != p2p::TransportMode::Tcp
        && let Ok(quic_addr) = "/ip4/0.0.0.0/udp/0/quic-v1".parse()
    {
        let _ = swarm.listen_on(quic_addr);
    }
    if mode != p2p::TransportMode::Quic
        && let Ok(tcp_addr) = "/ip4/0.0.0.0/tcp/0".parse()
    {
        let _ = swarm.listen_on(tcp_addr);
    }

    let own_peer_id = net_id::PeerId(swarm.local_peer_id().to_string());
    let mut listen_addrs: Vec<net_id::PeerAddr> = Vec::new();
    let mut ready_deadline: Option<tokio::time::Instant> = None;
    let mut mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
    let mut room_queries: std::collections::HashMap<libp2p::kad::QueryId, net_id::RoomName> =
        std::collections::HashMap::new();

    loop {
        let now = tokio::time::Instant::now();
        let far = now.checked_add(Duration::from_secs(3600)).unwrap_or(now);
        let ready_sleep = tokio::time::sleep_until(ready_deadline.unwrap_or(far));
        tokio::pin!(ready_sleep);
        let mesh_sleep = tokio::time::sleep_until(mesh_deadline.unwrap_or(far));
        tokio::pin!(mesh_sleep);
        tokio::select! {
            () = &mut ready_sleep, if ready_deadline.is_some() => {
                let addrs = std::mem::take(&mut listen_addrs);
                let me = net_id::Peer { id: own_peer_id.clone(), addrs };
                event_tx.send(p2p::Event::Net(p2p::NetEvent::Ready(me))).ok();
                ready_deadline = None;
            }
            () = &mut mesh_sleep => {
                p2p::swarm_loop::log_mesh_snapshot(&swarm);
                mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
            }
            cmd = cmd_rx.recv() => {
                if !p2p::swarm_loop::dispatch_command(&mut swarm, &event_tx, &mut room_queries, cmd) {
                    break;
                }
            }
            net_cmd = net_rx.recv() => {
                if let Some(net) = net_cmd {
                    p2p::swarm_loop::dispatch_net_command(&mut swarm, &event_tx, &mut room_queries, net);
                } else {
                    break;
                }
            }
            event = swarm.select_next_some() => {
                p2p::swarm_loop::handle_swarm_event(
                    &mut swarm,
                    &event_tx,
                    &room_queries,
                    &mut listen_addrs,
                    &mut ready_deadline,
                    mode,
                    event,
                );
            }
        }
    }
}

// no test_usage necessary
