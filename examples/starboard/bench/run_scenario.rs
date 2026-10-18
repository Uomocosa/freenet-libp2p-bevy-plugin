use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{identity, p2p, room_sync};
use tokio::sync::broadcast;

use crate::starboard;

// needed helper: everything one run records while the taps are drained
#[derive(Default)]
struct TapTally {
    pending: BTreeMap<(starboard::canvas::CellPos, starboard::canvas::CellPos), Instant>,
    samples: Vec<Duration>,
    received_bytes: Vec<u64>,
    arrived: BTreeSet<(
        usize,
        (starboard::canvas::CellPos, starboard::canvas::CellPos),
    )>,
}

#[rustfmt::skip]
impl TapTally {
    fn for_peers(peers: usize) -> Self {
        Self { received_bytes: vec![0; peers], ..Self::default() }
    }
}

pub async fn run_scenario(
    config: starboard::bench::ScenarioConfig,
) -> starboard::bench::BenchReport {
    let game = identity::GameId::from(starboard::bench::BENCH_GAME);
    let room = identity::RoomName::from(starboard::bench::BENCH_ROOM);
    let topic = room_sync::sync_topic(&game, &room);
    let mut apps: Vec<App> = (0..config.peers)
        .map(|_| starboard::app::test_harness::node_with(&game))
        .collect();
    let peers: Vec<identity::Peer> = apps
        .iter_mut()
        .map(starboard::app::test_harness::wait_ready)
        .collect();
    for (index, app) in apps.iter_mut().enumerate() {
        for (other, peer) in peers.iter().enumerate() {
            if other < index {
                starboard::app::test_harness::dial(app, peer);
            }
        }
    }
    for (index, app) in apps.iter_mut().enumerate() {
        let others = peers
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index)
            .map(|(_, peer)| peer.id.clone())
            .collect();
        starboard::app::test_harness::join(app, &room, others);
        app.world_mut()
            .resource_mut::<starboard::input::OwnBrush>()
            .size_index = starboard::bench::BENCH_SIZE_INDEX;
    }
    let peer_ids: Vec<identity::PeerId> = peers.iter().map(|peer| peer.id.clone()).collect();
    let mut taps: Vec<broadcast::Receiver<p2p::NetEvent>> = apps
        .iter()
        .map(|app| app.world().resource::<p2p::NetEventChannel>().subscribe())
        .collect();

    let warmup_frames = config
        .ops_per_second
        .saturating_mul(starboard::bench::BENCH_WARMUP_SECONDS);
    let mut discarded = TapTally::for_peers(config.peers);
    for tick in 0..warmup_frames {
        paint_frame(
            &mut apps,
            &mut taps,
            &topic,
            &peer_ids,
            config.peers,
            tick,
            &mut discarded,
        );
        tokio::time::sleep(starboard::bench::BENCH_FRAME).await;
    }

    let frames = config.ops_per_second.saturating_mul(config.seconds);
    let mut tally = TapTally::for_peers(config.peers);
    let mut sent_ops: u32 = 0;
    let measured_start = Instant::now();
    for tick in 0..frames {
        let frame_start = Instant::now();
        paint_frame(
            &mut apps,
            &mut taps,
            &topic,
            &peer_ids,
            config.peers,
            warmup_frames.saturating_add(tick),
            &mut tally,
        );
        sent_ops = sent_ops.saturating_add(u32::try_from(config.peers).unwrap_or(0));
        let deadline = frame_start.checked_add(starboard::bench::BENCH_FRAME);
        loop {
            drain_taps(&mut taps, &topic, &peer_ids, &mut tally);
            if deadline.is_none_or(|deadline| Instant::now() >= deadline) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    }
    let measured = measured_start.elapsed();

    let stopped_at = Instant::now();
    let convergence = loop {
        for app in &mut apps {
            app.update();
        }
        drain_taps(&mut taps, &topic, &peer_ids, &mut tally);
        let hashes: Vec<u64> = apps
            .iter()
            .map(|app| {
                starboard::canvas::hash_canvas(app.world().resource::<starboard::canvas::Canvas>())
            })
            .collect();
        if hashes.windows(2).all(|pair| pair.first() == pair.last()) {
            break stopped_at.elapsed();
        }
        assert!(
            stopped_at.elapsed() <= starboard::bench::BENCH_CONVERGENCE_BUDGET,
            "canvases did not converge: {hashes:?}"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    };

    let announcement_bytes = apps.first().map_or(0, |app| {
        let canvas = app.world().resource::<starboard::canvas::Canvas>();
        let message = room_sync::ReplicaSyncTopicMessage::<starboard::canvas::DrawOp>::CompareSliceContentHashes {
            room: room.clone(),
            hashes: room_sync::RoomState::slice_content_hashes(canvas),
        };
        bincode::serialized_size(&message)
            .ok()
            .and_then(|size| usize::try_from(size).ok())
            .unwrap_or(0)
    });

    let measured_millis = u32::try_from(measured.as_millis().min(600_000)).unwrap_or(600_000);
    let measured_seconds = f32::from(u16::try_from(measured_millis / 1000).unwrap_or(600));
    let total_bytes = tally
        .received_bytes
        .iter()
        .copied()
        .fold(0_u64, u64::saturating_add);
    let total_pairs = u64::from(sent_ops)
        .saturating_mul(u64::try_from(config.peers.saturating_sub(1)).unwrap_or(0));
    let arrived_pairs = u64::try_from(tally.arrived.len()).unwrap_or(u64::MAX);
    let dropped_pairs = total_pairs.saturating_sub(arrived_pairs);
    let dropped_per_mille = if total_pairs == 0 {
        0
    } else {
        u32::try_from(dropped_pairs.saturating_mul(1000) / total_pairs).unwrap_or(1000)
    };
    starboard::bench::BenchReport {
        config,
        achieved_ops_per_second: f32::from(
            u16::try_from(sent_ops.min(u32::from(u16::MAX))).unwrap_or(u16::MAX),
        ) / measured_seconds.max(1.0)
            / f32::from(u16::try_from(config.peers.max(1)).unwrap_or(1)),
        latency: starboard::bench::percentiles(&tally.samples),
        received_bytes_per_second: f64::from(
            u32::try_from(total_bytes.min(u64::from(u32::MAX))).unwrap_or(u32::MAX),
        ) / (f64::from(measured_millis) / 1000.0).max(1.0),
        dropped_percent: f32::from(
            u16::try_from(dropped_per_mille.min(u32::from(u16::MAX))).unwrap_or(u16::MAX),
        ) / 10.0,
        convergence,
        announcement_bytes,
        per_peer_bytes: peer_ids
            .iter()
            .zip(tally.received_bytes.iter())
            .map(|(peer, bytes)| starboard::bench::PeerTraffic {
                peer: peer.clone(),
                received_bytes: *bytes,
            })
            .collect(),
    }
}

// needed helper: paint one frame on every peer, draining the taps after each update
fn paint_frame(
    apps: &mut [App],
    taps: &mut [broadcast::Receiver<p2p::NetEvent>],
    topic: &identity::Topic,
    peer_ids: &[identity::PeerId],
    peer_count: usize,
    tick: u32,
    tally: &mut TapTally,
) {
    for (index, app) in apps.iter_mut().enumerate() {
        let brush = *app.world().resource::<starboard::input::OwnBrush>();
        let op = starboard::bench::scripted_stroke(index, peer_count, tick, brush);
        tally.pending.insert((op.from, op.to), Instant::now());
        app.world_mut()
            .write_message(starboard::input::PendingLocalDraw(op));
        app.update();
        drain_taps(taps, topic, peer_ids, tally);
    }
}

// needed helper: one round of tap draining, recording bytes, latency samples and arrivals
fn drain_taps(
    taps: &mut [broadcast::Receiver<p2p::NetEvent>],
    topic: &identity::Topic,
    peer_ids: &[identity::PeerId],
    tally: &mut TapTally,
) {
    let now = Instant::now();
    for (receiver, tap) in taps.iter_mut().enumerate() {
        while let Ok(event) = tap.try_recv() {
            match event {
                p2p::NetEvent::Gossip {
                    topic: event_topic,
                    from,
                    data,
                } => {
                    if &event_topic != topic {
                        continue;
                    }
                    count_bytes(peer_ids, &mut tally.received_bytes, &from, data.len());
                    let Ok(message) = bincode::deserialize::<
                        room_sync::ReplicaSyncTopicMessage<starboard::canvas::DrawOp>,
                    >(&data) else {
                        continue;
                    };
                    if let room_sync::ReplicaSyncTopicMessage::ApplyThisOp { op, .. } = message {
                        let key = (op.from, op.to);
                        if let Some(sent_at) = tally.pending.get(&key) {
                            tally.samples.push(now.saturating_duration_since(*sent_at));
                            tally.arrived.insert((receiver, key));
                        }
                    }
                }
                p2p::NetEvent::SnapshotPart { from, data, .. }
                | p2p::NetEvent::ReplicaSlice { from, data, .. } => {
                    count_bytes(peer_ids, &mut tally.received_bytes, &from, data.len());
                }
                _ => {}
            }
        }
    }
}

// needed helper: add one inbound payload to its sender's byte total
fn count_bytes(
    peer_ids: &[identity::PeerId],
    received_bytes: &mut [u64],
    from: &identity::PeerId,
    bytes: usize,
) {
    let Some(sender) = peer_ids.iter().position(|peer| peer == from) else {
        return;
    };
    if let Some(slot) = received_bytes.get_mut(sender) {
        *slot = slot.saturating_add(u64::try_from(bytes).unwrap_or(0));
    }
}

// no test_usage necessary
