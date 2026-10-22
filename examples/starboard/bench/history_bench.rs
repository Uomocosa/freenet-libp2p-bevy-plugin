use std::time::{Duration, Instant};

use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::room_sync::RoomState;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, room_sync};

use crate::starboard;

/// The strokes the e2e board paints.
const E2E_STROKES: u32 = 6;
/// The strokes of the busy board: enough to cover the whole canvas.
const BUSY_STROKES: u32 = 10_000;
/// The bench never lets the plugin write on its own; it drives the write itself.
const HISTORY_WRITE_INTERVAL: Duration = Duration::from_secs(3600);
const HISTORY_RECORD_TTL: Duration = Duration::from_secs(90);
const HISTORY_PER_ROOM_BYTES: u64 = 64 * 1024 * 1024;
const SETTLE: Duration = Duration::from_millis(20);
const MAX_ROUNDS: usize = 400;

/// The board a history measurement runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryBoard {
    /// The board the e2e paints: a few scripted strokes.
    E2e,
    /// A busy board: thousands of strokes over the whole canvas.
    Busy,
}

impl HistoryBoard {
    /// The board's name, as the measurement report writes it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::E2e => "e2e board",
            Self::Busy => "busy board",
        }
    }

    /// The board's stroke count.
    #[must_use]
    pub const fn strokes(self) -> u32 {
        match self {
            Self::E2e => E2E_STROKES,
            Self::Busy => BUSY_STROKES,
        }
    }
}

/// The measured size and latency of one room's Kademlia history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryReport {
    pub board: HistoryBoard,
    pub strokes: u32,
    pub snapshot_bytes: u64,
    pub record_bytes: u64,
    pub chunk_count: u32,
    pub put_latency_ms: u128,
    pub get_latency_ms: u128,
}

/// Measures how big a room's history is and how long a put and a get take.
///
/// Two in-process peers on localhost, direct dial — the transport, not the
/// discovery (§0.8). The writer stores the room's snapshot through the plugin's
/// own writer function; the reader asks for the manifest over Kademlia.
pub async fn measure_history(board: HistoryBoard) -> HistoryReport {
    let game = identity::GameId::from(starboard::bench::BENCH_GAME);
    let room = identity::RoomName::from(starboard::bench::BENCH_ROOM);
    let history = room_sync::HistoryConfig {
        write_interval: HISTORY_WRITE_INTERVAL,
        record_ttl: HISTORY_RECORD_TTL,
        per_room_bytes: p2p::HistoryByteCount::from(HISTORY_PER_ROOM_BYTES),
        max_fetch_retries: room_sync::HistoryConfig::default().max_fetch_retries,
    };
    let mut writer = starboard::app::test_harness::node_with(&game, Some(history));
    let mut reader = starboard::app::test_harness::node_with(&game, Some(history));
    let peers: Vec<identity::Peer> = vec![
        starboard::app::test_harness::wait_ready(&mut writer),
        starboard::app::test_harness::wait_ready(&mut reader),
    ];
    starboard::app::test_harness::dial(&mut reader, &peers[0]);
    // The lobby seeds the kademlia routing table of every peer it knows; the
    // bench does the same, or a put has nobody to confirm its quorum.
    seed_kad(
        &mut writer,
        &starboard::app::test_harness::dialable(&peers[1]),
    );
    seed_kad(
        &mut reader,
        &starboard::app::test_harness::dialable(&peers[0]),
    );

    let prefix = p2p::HistoryKeyPrefix::new(&game, &room, discovery::UnixTime::from(1));
    paint(&mut writer, board);
    let (snapshot_bytes, commands) = write_commands(&mut writer, &prefix);
    let (record_bytes, chunk_count) = command_sizes(&commands);

    let mut writer_events = writer
        .world()
        .resource::<p2p::NetEventChannel>()
        .subscribe();
    let started = Instant::now();
    for command in commands {
        writer
            .world()
            .resource::<p2p::NetCommandSender>()
            .send(command)
            .ok();
    }
    let put_latency_ms = wait_for_manifest(&mut writer, &mut writer_events, started)
        .await
        .as_millis();

    let mut reader_events = reader
        .world()
        .resource::<p2p::NetEventChannel>()
        .subscribe();
    let started = Instant::now();
    reader
        .world()
        .resource::<p2p::NetCommandSender>()
        .send(p2p::NetCommand::GetHistoryRecord {
            id: p2p::RoomHistoryRecordKind::Manifest(prefix),
        })
        .ok();
    let get_latency_ms = wait_for_found(&mut reader, &mut reader_events, started)
        .await
        .as_millis();

    HistoryReport {
        board,
        strokes: board.strokes(),
        snapshot_bytes,
        record_bytes,
        chunk_count,
        put_latency_ms,
        get_latency_ms,
    }
}

/// The markdown of the history measurements.
#[must_use]
pub fn history_markdown(reports: &[HistoryReport]) -> String {
    let machine = starboard::bench::machine_info();
    let mut out = String::new();
    out.push_str("# Starboard history measurement\n\n");
    out.push_str("Generated by `just bench-history` — do not edit by hand.\n\n");
    out.push_str(&format!("- Date: {}\n", starboard::bench::bench_date()));
    out.push_str(&format!(
        "- Machine: {} · {} · {} threads\n",
        machine.kernel, machine.cpu, machine.cores
    ));
    out.push_str(
        "- In-process peers on localhost, direct dial — no Freenet discovery, no NAT/relay (§0.8)\n",
    );
    out.push_str("- The put latency is the time until a second peer confirmed the record; the get latency is the time until the reader held the manifest\n\n");
    out.push_str(
        "| Board | Strokes / tiles | Snapshot bytes | Record bytes | Chunks | Put latency | Get latency |\n",
    );
    out.push_str("|---|---:|---:|---:|---:|---:|---:|\n");
    for report in reports {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} ms | {} ms |\n",
            report.board.label(),
            report.strokes,
            report.snapshot_bytes,
            report.record_bytes,
            report.chunk_count,
            report.put_latency_ms,
            report.get_latency_ms,
        ));
    }
    out
}

// needed helper: tells one peer's kademlia about the other, the way the lobby does
fn seed_kad(app: &mut App, peer: &identity::Peer) {
    app.world()
        .resource::<p2p::NetCommandSender>()
        .send(p2p::NetCommand::AddKadPeer(peer.clone()))
        .ok();
}

// needed helper: paints the measured board on the writer's canvas
fn paint(app: &mut App, board: HistoryBoard) {
    let brush = *app.world().resource::<starboard::input::OwnBrush>();
    let mut canvas = app.world_mut().resource_mut::<starboard::canvas::Canvas>();
    for tick in 0..board.strokes() {
        let op = starboard::bench::scripted_stroke(0, 1, tick, brush);
        canvas.apply(
            room_sync::OpMeta {
                lamport: tick.saturating_add(1),
                author: 1,
            },
            &op,
        );
    }
}

// needed helper: the plugin's own writer output for the writer's canvas
fn write_commands(app: &mut App, prefix: &p2p::HistoryKeyPrefix) -> (u64, Vec<p2p::NetCommand>) {
    let mut canvas = app.world_mut().resource_mut::<starboard::canvas::Canvas>();
    let snapshot = room_sync::RoomState::snapshot(&*canvas);
    let snapshot_bytes = bincode::serialize(&snapshot).map_or(0, |bytes| bytes.len());
    let commands = p2p::NetCommand::merge_and_write_history_records(
        &mut *canvas,
        prefix,
        None,
        None,
        1,
        p2p::HistoryByteCount::from(HISTORY_PER_ROOM_BYTES),
        HISTORY_RECORD_TTL,
    );
    (u64::try_from(snapshot_bytes).unwrap_or(0), commands)
}

// needed helper: the total bytes and the chunk count of the write commands
fn command_sizes(commands: &[p2p::NetCommand]) -> (u64, u32) {
    let mut bytes = 0_u64;
    let mut chunks = 0_u32;
    for command in commands {
        if let p2p::NetCommand::PutHistoryRecord { id, value, .. } = command
            && matches!(id, p2p::RoomHistoryRecordKind::Chunk(_))
        {
            bytes = bytes.saturating_add(u64::try_from(value.len()).unwrap_or(0));
            chunks = chunks.saturating_add(1);
        }
    }
    (bytes, chunks)
}

// needed helper: pumps the writer until it stored the manifest
async fn wait_for_manifest(
    app: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::NetEvent>,
    started: Instant,
) -> Duration {
    for _ in 0..MAX_ROUNDS {
        app.update();
        while let Ok(event) = events.try_recv() {
            if let p2p::NetEvent::HistoryRecordStored { id } = &event
                && matches!(id, p2p::RoomHistoryRecordKind::Manifest(_))
            {
                return started.elapsed();
            }
        }
        tokio::time::sleep(SETTLE).await;
    }
    started.elapsed()
}

// needed helper: pumps the reader until the manifest arrived
async fn wait_for_found(
    app: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::NetEvent>,
    started: Instant,
) -> Duration {
    for _ in 0..MAX_ROUNDS {
        app.update();
        while let Ok(event) = events.try_recv() {
            if matches!(event, p2p::NetEvent::HistoryRecordFound { .. }) {
                return started.elapsed();
            }
        }
        tokio::time::sleep(SETTLE).await;
    }
    started.elapsed()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "benchmark: one room's Kademlia history, writes examples/starboard/HISTORY.md"]
async fn history_bench() {
    let boards = [HistoryBoard::E2e, HistoryBoard::Busy];
    let mut reports = Vec::new();
    for board in boards {
        let report = measure_history(board).await;
        println!(
            "starboard history board={} strokes={} snapshot={}B record={}B chunks={} put={}ms get={}ms",
            report.board.label(),
            report.strokes,
            report.snapshot_bytes,
            report.record_bytes,
            report.chunk_count,
            report.put_latency_ms,
            report.get_latency_ms,
        );
        reports.push(report);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    let markdown = history_markdown(&reports);
    let path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/starboard/HISTORY.md");
    std::fs::write(&path, markdown).expect("HISTORY.md written");
    println!("starboard history wrote {}", path.display());
}
