use std::time::Duration;

use futures::StreamExt;
use libp2p::identity::Keypair;

use crate::identity;
use crate::methods;
use crate::p2p;

pub async fn run<Payload: p2p::traits::Message>(
    config: &p2p::RunnerConfig,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
    mut libp2p_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Libp2pCommand>,
    event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    keypair: Keypair,
) {
    let mode = config.mode;
    let mut state = match methods::runner_config::build_swarm::<Payload>(config, keypair) {
        Ok(swarm) => p2p::SwarmLoopState::new(
            swarm,
            event_tx,
            p2p::swarm_loop::RoomSubscriptions::new(config.game.clone()),
            p2p::swarm_loop::PendingSnapshotReplies::default(),
            mode,
        ),
        Err(e) => {
            event_tx.send(p2p::Event::Error(e)).ok();
            return;
        }
    };

    if mode != p2p::TransportMode::Tcp
        && let Ok(quic_addr) = "/ip4/0.0.0.0/udp/0/quic-v1".parse()
    {
        let _ = state.swarm.listen_on(quic_addr);
    }
    if mode != p2p::TransportMode::Quic
        && let Ok(tcp_addr) = "/ip4/0.0.0.0/tcp/0".parse()
    {
        let _ = state.swarm.listen_on(tcp_addr);
    }

    let own_peer_id = identity::PeerId(state.swarm.local_peer_id().to_string());
    let mut mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));

    loop {
        let now = tokio::time::Instant::now();
        let far = now.checked_add(Duration::from_secs(3600)).unwrap_or(now);
        let ready_sleep = tokio::time::sleep_until(state.ready_deadline.unwrap_or(far));
        tokio::pin!(ready_sleep);
        let mesh_sleep = tokio::time::sleep_until(mesh_deadline.unwrap_or(far));
        tokio::pin!(mesh_sleep);
        tokio::select! {
            () = &mut ready_sleep, if state.ready_deadline.is_some() => {
                let addrs = std::mem::take(&mut state.listen_addrs);
                let me = identity::Peer { id: own_peer_id.clone(), addrs };
                state.event_tx.send(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(me))).ok();
                state.ready_deadline = None;
            }
            () = &mut mesh_sleep => {
                state.log_mesh_snapshot();
                mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
            }
            cmd = cmd_rx.recv() => {
                if !state.dispatch_command(&config.gossipsub, cmd) {
                    break;
                }
            }
            libp2p_cmd = libp2p_rx.recv() => {
                if let Some(libp2p_command) = libp2p_cmd {
                    state.dispatch_libp2p_command(libp2p_command);
                } else {
                    break;
                }
            }
            event = state.swarm.select_next_some() => {
                state.handle_swarm_event(event);
            }
        }
    }
}

// no test_usage necessary
