use std::time::Duration;

use futures::StreamExt;
use libp2p::identity::Keypair;

use crate::identity;
use crate::p2p;

pub async fn run<Payload: p2p::Message>(
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
    mut net_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::NetCommand>,
    event_tx: tokio::sync::mpsc::UnboundedSender<p2p::Event<Payload>>,
    keypair: Keypair,
    config: p2p::RunnerConfig,
) {
    let mode = config.mode;
    let mut swarm = match p2p::build_swarm::build_swarm::<Payload>(
        keypair,
        config.mdns,
        &config.game,
        &config.gossipsub,
    ) {
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

    let own_peer_id = identity::PeerId(swarm.local_peer_id().to_string());
    let mut listen_addrs: Vec<identity::PeerAddr> = Vec::new();
    let mut ready_deadline: Option<tokio::time::Instant> = None;
    let mut mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
    let mut subscriptions = p2p::swarm_loop::RoomSubscriptions::new(config.game);
    let mut snapshot_replies = p2p::swarm_loop::PendingSnapshotReplies::default();

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
                let me = identity::Peer { id: own_peer_id.clone(), addrs };
                event_tx.send(p2p::Event::Net(p2p::NetEvent::Ready(me))).ok();
                ready_deadline = None;
            }
            () = &mut mesh_sleep => {
                p2p::swarm_loop::log_mesh_snapshot(&swarm);
                mesh_deadline = tokio::time::Instant::now().checked_add(Duration::from_secs(30));
            }
            cmd = cmd_rx.recv() => {
                if !p2p::swarm_loop::dispatch_command(
                    &mut swarm,
                    &event_tx,
                    &mut subscriptions,
                    &mut snapshot_replies,
                    &config.gossipsub,
                    cmd,
                ) {
                    break;
                }
            }
            net_cmd = net_rx.recv() => {
                if let Some(net) = net_cmd {
                    p2p::swarm_loop::dispatch_net_command(&mut swarm, &event_tx, &mut subscriptions, &mut snapshot_replies, net);
                } else {
                    break;
                }
            }
            event = swarm.select_next_some() => {
                p2p::swarm_loop::handle_swarm_event(
                    &mut swarm,
                    &event_tx,
                    &subscriptions,
                    &mut snapshot_replies,
                    &mut listen_addrs,
                    &mut ready_deadline,
                    mode,
                    event,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use derive_more::Deref;
    use libp2p::identity::Keypair;
    use serde::{Deserialize, Serialize};

    use crate::identity;
    use crate::p2p;

    const SUBSCRIPTION_SETTLE: Duration = Duration::from_millis(500);

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
    struct TestPayload(u32);

    struct Node {
        commands: tokio::sync::mpsc::UnboundedSender<p2p::Command<TestPayload>>,
        net_commands: tokio::sync::mpsc::UnboundedSender<p2p::NetCommand>,
        events: tokio::sync::mpsc::UnboundedReceiver<p2p::Event<TestPayload>>,
        handle: tokio::task::JoinHandle<()>,
    }

    fn node(game: &identity::GameId) -> Node {
        let (commands, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let (net_commands, net_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, events) = tokio::sync::mpsc::unbounded_channel();
        let handle = p2p::spawn_runner::<TestPayload>(
            command_rx,
            net_rx,
            event_tx,
            Keypair::generate_ed25519(),
            p2p::RunnerConfig {
                mode: p2p::TransportMode::Tcp,
                mdns: p2p::MdnsMode::Disabled,
                game: game.clone(),
                gossipsub: p2p::GossipsubConfig::default(),
            },
        );
        Node {
            commands,
            net_commands,
            events,
            handle,
        }
    }

    fn dialable(peer: &identity::Peer) -> identity::Peer {
        identity::Peer {
            id: peer.id.clone(),
            addrs: peer
                .addrs
                .iter()
                .map(|addr| identity::PeerAddr(addr.replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/")))
                .collect(),
        }
    }

    async fn wait_ready(node: &mut Node) -> identity::Peer {
        let ready = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(event) = node.events.recv().await {
                if let p2p::Event::Net(p2p::NetEvent::Ready(peer)) = event {
                    return Some(peer);
                }
            }
            None
        })
        .await;
        let Ok(Some(peer)) = ready else {
            panic!("node did not become ready");
        };
        peer
    }

    async fn wait_connected_to_all(node: &mut Node, peers: &[identity::PeerId]) {
        let mut seen: Vec<identity::PeerId> = Vec::new();
        let connected = tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(event) = node.events.recv().await {
                if let p2p::Event::Net(p2p::NetEvent::PeerConnected(id)) = event
                    && peers.contains(&id)
                    && !seen.contains(&id)
                {
                    seen.push(id);
                    if seen.len() == peers.len() {
                        return true;
                    }
                }
            }
            false
        })
        .await;
        assert_eq!(connected, Ok(true));
    }

    async fn wait_room_message(
        node: &mut Node,
        room: &identity::RoomName,
    ) -> p2p::Event<TestPayload> {
        let message = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(event) = node.events.recv().await {
                if matches!(&event, p2p::Event::RoomMessage { room: seen, .. } if seen == room) {
                    return Some(event);
                }
            }
            None
        })
        .await;
        let Ok(Some(message)) = message else {
            panic!("room message did not arrive");
        };
        message
    }

    async fn count_room_messages(
        node: &mut Node,
        room: &identity::RoomName,
        wait: Duration,
    ) -> usize {
        let mut count = 0usize;
        let sleep = tokio::time::sleep(wait);
        tokio::pin!(sleep);
        loop {
            tokio::select! {
                () = &mut sleep => return count,
                event = node.events.recv() => {
                    let Some(event) = event else {
                        return count;
                    };
                    if matches!(&event, p2p::Event::RoomMessage { room: seen, .. } if seen == room) {
                        count = count.saturating_add(1);
                    }
                }
            }
        }
    }

    #[test]
    fn test_usage() {
        let _ = stringify!(run::<TestPayload>);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn transport_only_three_swarms_receive_a_room_payload_exactly_once() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let mut hub = node(&game);
        let mut left = node(&game);
        let mut right = node(&game);

        for node in [&mut hub, &mut left, &mut right] {
            node.net_commands
                .send(p2p::NetCommand::SubscribeRoom { room: room.clone() })
                .unwrap();
        }

        let hub_peer = wait_ready(&mut hub).await;
        let left_peer = wait_ready(&mut left).await;
        let right_peer = wait_ready(&mut right).await;

        left.net_commands
            .send(p2p::NetCommand::Dial(dialable(&hub_peer)))
            .unwrap();
        right
            .net_commands
            .send(p2p::NetCommand::Dial(dialable(&hub_peer)))
            .unwrap();

        wait_connected_to_all(&mut hub, &[left_peer.id.clone(), right_peer.id.clone()]).await;
        wait_connected_to_all(&mut left, std::slice::from_ref(&hub_peer.id)).await;
        wait_connected_to_all(&mut right, std::slice::from_ref(&hub_peer.id)).await;

        tokio::time::sleep(SUBSCRIPTION_SETTLE).await;

        hub.commands
            .send(p2p::Command::PublishInJoinedRoom {
                payload: TestPayload(42),
            })
            .unwrap();

        for node in [&mut left, &mut right] {
            let event = wait_room_message(node, &room).await;
            let p2p::Event::RoomMessage {
                room: seen,
                from,
                payload,
            } = event
            else {
                panic!("expected a room message");
            };
            assert_eq!(seen, room);
            assert_eq!(from, hub_peer.id);
            assert_eq!(payload, TestPayload(42));
            assert_eq!(
                count_room_messages(node, &room, Duration::from_millis(500)).await,
                0
            );
        }

        hub.handle.abort();
        left.handle.abort();
        right.handle.abort();
    }
}
