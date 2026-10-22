use std::time::Duration;

use futures::StreamExt;
use libp2p::identity::Keypair;

use crate::identity;
use crate::methods;
use crate::p2p;

pub async fn run<Payload: p2p::Message>(
    config: &p2p::RunnerConfig,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::Command<Payload>>,
    mut net_rx: tokio::sync::mpsc::UnboundedReceiver<p2p::NetCommand>,
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
                state.event_tx.send(p2p::Event::Net(p2p::NetEvent::Ready(me))).ok();
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
            net_cmd = net_rx.recv() => {
                if let Some(net) = net_cmd {
                    state.dispatch_net_command(net);
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use derive_more::Deref;
    use libp2p::identity::Keypair;
    use serde::{Deserialize, Serialize};

    use crate::discovery;
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
        let config = p2p::RunnerConfig {
            mode: p2p::TransportMode::Tcp,
            mdns: p2p::MdnsMode::Disabled,
            game: game.clone(),
            gossipsub: p2p::GossipsubConfig::default(),
        };
        let handle = config.spawn_runner::<TestPayload>(
            command_rx,
            net_rx,
            event_tx,
            Keypair::generate_ed25519(),
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

    async fn wait_history_event(node: &mut Node) -> p2p::NetEvent {
        let event = tokio::time::timeout(Duration::from_secs(15), async {
            while let Some(event) = node.events.recv().await {
                if let p2p::Event::Net(
                    net @ (p2p::NetEvent::HistoryRecordStored { .. }
                    | p2p::NetEvent::HistoryRecordPutFailed { .. }
                    | p2p::NetEvent::HistoryRecordFound { .. }
                    | p2p::NetEvent::HistoryRecordNotFound { .. }
                    | p2p::NetEvent::HistoryStoreMeasured { .. }),
                ) = event
                {
                    return Some(net);
                }
            }
            None
        })
        .await;
        let Ok(Some(event)) = event else {
            panic!("no history record event arrived");
        };
        event
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

    #[tokio::test(flavor = "multi_thread")]
    async fn kademlia_history_record_survives_its_writer_leaving() {
        let game = identity::GameId::from("game");
        let mut writer = node(&game);
        let mut middle = node(&game);
        let mut reader = node(&game);
        let writer_peer = wait_ready(&mut writer).await;
        let middle_peer = dialable(&wait_ready(&mut middle).await);
        let reader_peer = wait_ready(&mut reader).await;

        for node in [&writer, &reader] {
            node.net_commands
                .send(p2p::NetCommand::AddKadPeer(middle_peer.clone()))
                .unwrap();
            node.net_commands
                .send(p2p::NetCommand::Dial(middle_peer.clone()))
                .unwrap();
        }
        wait_connected_to_all(
            &mut middle,
            &[writer_peer.id.clone(), reader_peer.id.clone()],
        )
        .await;

        let prefix = p2p::HistoryKeyPrefix::from("room-generation");
        let manifest = p2p::RoomHistoryRecordKind::Manifest(prefix.clone());
        let chunk = p2p::RoomHistoryRecordKind::Chunk(p2p::HistoryChunkId {
            prefix,
            content_hash: p2p::ContentHash::from("hash"),
            chunk: p2p::ChunkIndex(0),
        });
        let chunk_value: Vec<u8> = (0..60 * 1024).map(|i: u32| i.to_le_bytes()[0]).collect();
        for (id, value) in [(&manifest, vec![7, 7, 7]), (&chunk, chunk_value.clone())] {
            writer
                .net_commands
                .send(p2p::NetCommand::PutHistoryRecord {
                    id: id.clone(),
                    value,
                    ttl: Duration::from_secs(90),
                })
                .unwrap();
            assert_eq!(
                wait_history_event(&mut writer).await,
                p2p::NetEvent::HistoryRecordStored { id: id.clone() }
            );
        }
        writer.handle.abort();

        for (id, value) in [(&manifest, vec![7, 7, 7]), (&chunk, chunk_value)] {
            reader
                .net_commands
                .send(p2p::NetCommand::GetHistoryRecord { id: id.clone() })
                .unwrap();
            assert_eq!(
                wait_history_event(&mut reader).await,
                p2p::NetEvent::HistoryRecordFound {
                    id: id.clone(),
                    value
                }
            );
        }

        let unknown = p2p::RoomHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("nope"));
        reader
            .net_commands
            .send(p2p::NetCommand::GetHistoryRecord {
                id: unknown.clone(),
            })
            .unwrap();
        assert_eq!(
            wait_history_event(&mut reader).await,
            p2p::NetEvent::HistoryRecordNotFound { id: unknown }
        );

        middle.handle.abort();
        reader.handle.abort();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn history_over_limit_is_removed_from_the_kademlia_store() {
        let game = identity::GameId::from("game");
        let mut holder = node(&game);
        wait_ready(&mut holder).await;
        let mut state = discovery::state_machine::DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.game = game.clone();
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(1);
        let room = identity::RoomName::from("old-room");
        let created_at = discovery::UnixTime::from(7);
        state.held_rooms.insert(
            room.clone(),
            discovery::state_machine::HeldRoomHistory {
                created_at,
                left_at: discovery::UnixTime::from(8),
            },
        );
        let prefix = p2p::HistoryKeyPrefix::new(&game, &room, created_at);
        let manifest = p2p::RoomHistoryRecordKind::Manifest(prefix.clone());
        holder
            .net_commands
            .send(p2p::NetCommand::PutHistoryRecord {
                id: manifest,
                value: vec![1; 32],
                ttl: Duration::from_secs(90),
            })
            .unwrap();
        wait_history_event(&mut holder).await;

        holder
            .net_commands
            .send(p2p::NetCommand::MeasureHistoryStore)
            .unwrap();
        let p2p::NetEvent::HistoryStoreMeasured { bytes_by_prefix } =
            wait_history_event(&mut holder).await
        else {
            panic!("expected a history store measurement");
        };
        assert_eq!(
            bytes_by_prefix.get(&prefix),
            Some(&p2p::HistoryByteCount(32))
        );
        for output in state.drop_oldest_history(&bytes_by_prefix) {
            if let discovery::state_machine::Output::NetCommand(command) = output {
                holder.net_commands.send(command).unwrap();
            }
        }
        assert_eq!(state.evicted_rooms.get(&room), Some(&created_at));

        holder
            .net_commands
            .send(p2p::NetCommand::MeasureHistoryStore)
            .unwrap();
        assert_eq!(
            wait_history_event(&mut holder).await,
            p2p::NetEvent::HistoryStoreMeasured {
                bytes_by_prefix: std::collections::BTreeMap::new()
            }
        );
        holder.handle.abort();
    }
}
