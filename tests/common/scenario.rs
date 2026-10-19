use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use telegram_bot_helper::send_video_best_effort;

use super::{
    CanvasTick, TerminalGuard, build_game, canvas_ticks, cleanup_stale, finish_record, last_canvas,
    last_tick, log_contains, marker_times, poke, read_head, require_x11, rings_ready,
    senders_seen_in_room, speed_clip, start_record, tile, wakeup_screen,
};

pub const DISCOVERY_BUDGET_SECS: u64 = 120;
pub const JOIN_BUDGET_SECS: u64 = 180;
pub const ROOM_MESSAGE_BUDGET_SECS: u64 = 30;
/// libp2p-only join target: last joiner's press -> every peer fully meshed.
pub const CONVERGE_BUDGET_MS: u64 = 500;
/// Starboard only: last scripted stroke -> every peer reporting the same canvas.
pub const CANVAS_BUDGET_SECS: u64 = 30;

const SPAWN_TIMEOUT_SECS: u64 = 180;
const RECORD_SECS: u64 = 600;
const POLL_MS: u64 = 500;
const SPAWN_GAP_MS: u64 = 300;
const CREATED_MARKER: &str = "lobby created room=";

pub struct Scenario {
    pub name: String,
    pub bin: &'static str,
    pub window_prefix: &'static str,
    pub peers: usize,
    pub join: bool,
    pub is_room_ping_enabled: bool,
    pub ui: Option<String>,
    pub scripts: Vec<Option<String>>,
    pub late_join: bool,
    pub discovery_budget_secs: u64,
    pub join_budget_secs: u64,
}

impl Scenario {
    #[must_use]
    pub fn discover_only(peers: usize) -> Self {
        Self {
            name: "lobby_room_discovery".to_string(),
            bin: "lobby_room",
            window_prefix: "lobby",
            peers,
            join: false,
            is_room_ping_enabled: false,
            ui: Some("custom".to_string()),
            scripts: Vec::new(),
            late_join: false,
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }

    #[must_use]
    pub fn join(peers: usize) -> Self {
        Self {
            name: "lobby_room_join".to_string(),
            bin: "lobby_room",
            window_prefix: "lobby",
            peers,
            join: true,
            is_room_ping_enabled: false,
            ui: Some("default".to_string()),
            scripts: Vec::new(),
            late_join: false,
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }

    #[must_use]
    pub fn room_messaging(peers: usize) -> Self {
        Self {
            name: "room_messaging".to_string(),
            bin: "lobby_room",
            window_prefix: "lobby",
            peers,
            join: true,
            is_room_ping_enabled: true,
            ui: Some("default".to_string()),
            scripts: Vec::new(),
            late_join: false,
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }

    /// Three starboard windows: p1 paints, p2 erases some of it, p3 only watches.
    #[must_use]
    pub fn starboard(peers: usize) -> Self {
        Self {
            name: "starboard_e2e".to_string(),
            bin: "starboard_gossip",
            window_prefix: "starboard",
            peers,
            join: true,
            is_room_ping_enabled: false,
            ui: None,
            scripts: vec![Some("paint:6".to_string()), Some("erase:3".to_string())],
            late_join: false,
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }

    /// The same, plus one peer that joins only after the painting stopped.
    #[must_use]
    pub fn starboard_late_join(peers: usize) -> Self {
        Self {
            late_join: true,
            ..Self::starboard(peers)
        }
    }
}

pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

pub struct Report {
    pub checks: Vec<Check>,
    pub caption: String,
    pub logs_dir: PathBuf,
}

impl Report {
    pub fn assert_all(&self) {
        for check in &self.checks {
            assert!(
                check.ok,
                "check `{}` failed: {} (logs: {})",
                check.name,
                check.detail,
                self.logs_dir.display()
            );
        }
    }
}

struct Env {
    bin: PathBuf,
    dir: PathBuf,
    token: String,
    room: String,
    transport: String,
    raw: PathBuf,
    clip: PathBuf,
    build_secs: f64,
    recording: Option<Child>,
}

struct Peers {
    windows: Vec<TerminalGuard>,
    logs: Vec<PathBuf>,
    room: String,
    created: bool,
    spawn_ok: bool,
    t_create: Instant,
    t_join: Instant,
}

pub async fn run_scenario(scenario: Scenario) -> Report {
    let mut env = match prepare(&scenario) {
        Ok(env) => env,
        Err(detail) => return early_report(&scenario, detail),
    };
    let mut peers = spawn(&scenario, &env);
    let checks = evaluate(&scenario, &env, &mut peers).await;
    let clip = stop_recording(env.recording.take(), &env.raw, &env.clip);
    let windows = std::mem::take(&mut peers.windows);
    release(windows);
    let caption = build_caption(&scenario, &env, &peers, &checks);
    eprintln!("{caption}");
    send_clip(clip.as_deref(), &caption).await;
    Report {
        checks,
        caption,
        logs_dir: env.dir,
    }
}

fn prepare(scenario: &Scenario) -> Result<Env, String> {
    wakeup_screen();
    require_x11()?;
    cleanup_stale();
    let build_start = Instant::now();
    let bin = build_game(scenario.bin)?;
    let build_secs = build_start.elapsed().as_secs_f64();
    let stamp = epoch_secs();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".local-run")
        .join(format!("{}-{stamp}", scenario.name));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let raw = dir.join("raw.mp4");
    let clip = dir.join("clip.mp4");
    let recording = start_record(RECORD_SECS, &raw);
    Ok(Env {
        bin,
        dir,
        token: format!("lobby-e2e/{stamp}"),
        room: format!("room-{stamp}"),
        transport: std::env::var("LOBBY_TRANSPORT").unwrap_or_else(|_| "both".to_string()),
        raw,
        clip,
        build_secs,
        recording,
    })
}

fn spawn(scenario: &Scenario, env: &Env) -> Peers {
    let mut windows = Vec::new();
    let mut logs = Vec::new();
    let create = format!("create:{}", env.room);
    let first = push_peer(
        scenario,
        env,
        0,
        "p1",
        Some(&create),
        &mut windows,
        &mut logs,
    );
    if let Err(e) = &first {
        eprintln!("{e}");
    }
    let mut spawn_ok = first.is_ok();
    // the UI picks the room name; read it back from the creator's log
    let room = logs
        .first()
        .filter(|log| did_log_appear_within(log, CREATED_MARKER, SPAWN_TIMEOUT_SECS))
        .and_then(|log| created_room(log))
        .unwrap_or_default();
    let created = !room.is_empty();
    let t_create = Instant::now();
    for index in 1..first_batch_size(scenario) {
        let username = format!("p{}", index.saturating_add(1));
        let action = scenario.join.then(|| format!("join:{room}"));
        if let Err(e) = push_peer(
            scenario,
            env,
            index,
            &username,
            action.as_deref(),
            &mut windows,
            &mut logs,
        ) {
            eprintln!("{e}");
            spawn_ok = false;
        }
        std::thread::sleep(Duration::from_millis(SPAWN_GAP_MS));
    }
    let t_join = Instant::now();
    if let Err(e) = tile(&windows) {
        eprintln!("tile: {e}");
    }
    Peers {
        windows,
        logs,
        room,
        created,
        spawn_ok,
        t_create,
        t_join,
    }
}

// needed helper: peers spawned before the checks run (the late joiner comes later)
const fn first_batch_size(scenario: &Scenario) -> usize {
    if scenario.late_join {
        scenario.peers.saturating_sub(1)
    } else {
        scenario.peers
    }
}

fn push_peer(
    scenario: &Scenario,
    env: &Env,
    index: usize,
    username: &str,
    action: Option<&str>,
    windows: &mut Vec<TerminalGuard>,
    logs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let log = spawn_peer(scenario, env, index, username, action, windows)?;
    logs.push(log);
    Ok(())
}

async fn evaluate(scenario: &Scenario, env: &Env, peers: &mut Peers) -> Vec<Check> {
    // Wait for the lobby row itself, not a transport-connection proxy. `ring ready`
    // is emitted once a node actually reads the lobby, which on mainnet can take
    // tens of seconds; the two checks then separate "a node never read the lobby"
    // (ring) from "lobby read but the row is missing" (discovery).
    let seen =
        did_predicate_hold_within(Duration::from_secs(scenario.discovery_budget_secs), || {
            do_all_peers_see_room(&peers.logs, &peers.room)
        })
        .await;
    let ring_ready = rings_ready(&peers.logs);
    let ring_ok = ring_ready.iter().all(|ready| *ready);
    let discovery_ms = peers.t_create.elapsed().as_millis();
    let discovery_ok = ring_ok
        && peers.created
        && peers.spawn_ok
        && seen
        && peers.t_create.elapsed().as_secs() <= scenario.discovery_budget_secs;
    let (join_ok, join_ms, join_detail) = evaluate_join(scenario, peers, ring_ok).await;
    let expected = first_batch_size(scenario);
    let spawn_check = Check {
        name: "spawn".to_string(),
        ok: peers.spawn_ok && peers.logs.len() == expected,
        detail: format!("{}/{} instances", peers.logs.len(), expected),
    };
    let published = creator_published(&peers.logs);
    let lobby_check = Check {
        name: "lobby".to_string(),
        ok: peers.created && published,
        detail: format!("created={} published={published}", peers.created),
    };
    let ring_check = Check {
        name: "ring".to_string(),
        ok: ring_ok,
        detail: ring_detail(&peers.logs, &ring_ready),
    };
    let discovery_check = Check {
        name: "discovery".to_string(),
        ok: discovery_ok,
        detail: format!(
            "{discovery_ms}ms budget={}s created={} seen={seen} isolated=[{}]",
            scenario.discovery_budget_secs,
            peers.created,
            isolated_peers(&peers.logs).join(",")
        ),
    };
    let join_check = Check {
        name: "join".to_string(),
        ok: join_ok,
        detail: format!(
            "{join_ms}ms budget={}s {join_detail}",
            scenario.join_budget_secs
        ),
    };
    let mut checks = vec![
        spawn_check,
        lobby_check,
        ring_check,
        discovery_check,
        join_check,
    ];
    if scenario.join {
        checks.push(converge_check(&peers.logs, peers.logs.len()));
    }
    if scenario.is_room_ping_enabled {
        checks.push(evaluate_room_messages(peers, join_ok).await);
    }
    if scenario.late_join {
        checks.push(late_join_check(scenario, env, peers).await);
    }
    if scenario.bin == "starboard_gossip" {
        checks.push(canvas_check(peers).await);
    }
    checks
}

// needed helper: spawns the late joiner once the others are meshed and waits for the new mesh
async fn late_join_check(scenario: &Scenario, env: &Env, peers: &mut Peers) -> Check {
    let index = peers.logs.len();
    let username = format!("p{}", index.saturating_add(1));
    let action = scenario.join.then(|| format!("join:{}", peers.room));
    let start = Instant::now();
    let pushed = push_peer(
        scenario,
        env,
        index,
        &username,
        action.as_deref(),
        &mut peers.windows,
        &mut peers.logs,
    );
    if let Err(e) = &pushed {
        eprintln!("{e}");
    }
    let spawned = pushed.is_ok();
    if let Err(e) = tile(&peers.windows) {
        eprintln!("tile: {e}");
    }
    let total = peers.logs.len();
    let meshed = did_predicate_hold_within(Duration::from_secs(scenario.join_budget_secs), || {
        are_all_peers_in_room(&peers.logs, &peers.room, total)
    })
    .await;
    let detail = if meshed {
        "full mesh".to_string()
    } else {
        format!("mesh not reached: {}", mesh_snapshot(&peers.logs, total))
    };
    Check {
        name: "late_join".to_string(),
        ok: spawned && meshed,
        detail: format!(
            "{}ms budget={}s {total} instances {detail}",
            start.elapsed().as_millis(),
            scenario.join_budget_secs
        ),
    }
}

// needed helper: every peer's canvas hash agrees after the last scripted stroke
async fn canvas_check(peers: &Peers) -> Check {
    let start = Instant::now();
    let agreed = did_predicate_hold_within(Duration::from_secs(CANVAS_BUDGET_SECS), || {
        do_canvases_agree(&peers.logs)
    })
    .await;
    let snapshot: Vec<String> = peers
        .logs
        .iter()
        .enumerate()
        .map(|(index, log)| {
            let tick = last_canvas(log);
            format!(
                "p{}:hash={},tiles={}",
                index.saturating_add(1),
                tick.as_ref().map_or(0, |tick| tick.hash),
                tick.as_ref().map_or(0, |tick| tick.tiles)
            )
        })
        .collect();
    Check {
        name: "canvas".to_string(),
        ok: agreed,
        detail: format!(
            "{}ms budget={CANVAS_BUDGET_SECS}s [{}]",
            start.elapsed().as_millis(),
            snapshot.join(" ")
        ),
    }
}

// needed helper: every peer reports the same non-empty canvas after the last stroke
fn do_canvases_agree(logs: &[PathBuf]) -> bool {
    let last_script = logs
        .iter()
        .filter_map(|log| {
            marker_times(log, "starboard script done")
                .last()
                .map(|(ms, _)| *ms)
        })
        .max();
    let Some(last_script) = last_script else {
        return false;
    };
    let seen: Vec<Option<CanvasTick>> = logs
        .iter()
        .map(|log| {
            canvas_ticks(log)
                .into_iter()
                .rfind(|(ms, _)| *ms >= last_script)
                .map(|(_, tick)| tick)
        })
        .collect();
    seen.first().and_then(Option::as_ref).is_some_and(|first| {
        first.tiles > 0
            && seen.iter().all(|tick| {
                tick.as_ref()
                    .is_some_and(|tick| tick.hash == first.hash && tick.tiles == first.tiles)
            })
    })
}

// needed helper: verifies every peer received a room message from every other peer
async fn evaluate_room_messages(peers: &Peers, join_ok: bool) -> Check {
    if !join_ok {
        return Check {
            name: "room_message".to_string(),
            ok: false,
            detail: "skipped: join did not reach full mesh".to_string(),
        };
    }
    let start = Instant::now();
    let want = peers.logs.len().saturating_sub(1);
    let delivered =
        did_predicate_hold_within(Duration::from_secs(ROOM_MESSAGE_BUDGET_SECS), || {
            peers
                .logs
                .iter()
                .all(|log| senders_seen_in_room(log, &peers.room).len() >= want)
        })
        .await;
    let counts: Vec<String> = peers
        .logs
        .iter()
        .enumerate()
        .map(|(index, log)| {
            let got = senders_seen_in_room(log, &peers.room).len();
            format!("p{}:{got}/{want}", index.saturating_add(1))
        })
        .collect();
    Check {
        name: "room_message".to_string(),
        ok: delivered,
        detail: format!(
            "{}ms budget={ROOM_MESSAGE_BUDGET_SECS}s [{}]",
            start.elapsed().as_millis(),
            counts.join(" ")
        ),
    }
}

async fn evaluate_join(scenario: &Scenario, peers: &Peers, ring_ok: bool) -> (bool, u128, String) {
    if !scenario.join {
        return (true, 0, "n/a".to_string());
    }
    if !ring_ok {
        return (
            false,
            peers.t_join.elapsed().as_millis(),
            "skipped: freenet ring never ready".to_string(),
        );
    }
    let mesh = did_predicate_hold_within(Duration::from_secs(scenario.join_budget_secs), || {
        are_all_peers_in_room(&peers.logs, &peers.room, peers.logs.len())
    })
    .await;
    let budget_ok = peers.t_join.elapsed().as_secs() <= scenario.join_budget_secs;
    let detail = if mesh {
        "full mesh".to_string()
    } else {
        format!(
            "mesh not reached: {}",
            mesh_snapshot(&peers.logs, peers.logs.len())
        )
    };
    (
        mesh && budget_ok,
        peers.t_join.elapsed().as_millis(),
        detail,
    )
}

fn stop_recording(recording: Option<Child>, raw: &Path, clip: &Path) -> Option<PathBuf> {
    recording
        .and_then(|child| finish_record(child, raw))
        .and_then(|raw| speed_clip(&raw, clip))
}

fn release(windows: Vec<TerminalGuard>) {
    if std::env::var("LOBBY_KEEP").is_err() {
        drop(windows);
    } else {
        std::mem::forget(windows);
    }
}

fn build_caption(scenario: &Scenario, env: &Env, peers: &Peers, checks: &[Check]) -> String {
    let mut lines = vec![
        freenet_status(env, &peers.logs),
        format!(
            "{} bin={} ui={} room={} transport={} peers={} · build {:.1}s",
            scenario.name,
            scenario.bin,
            scenario.ui.as_deref().unwrap_or("none"),
            peers.room,
            env.transport,
            peers.logs.len(),
            env.build_secs
        ),
        format!("created={} spawn_ok={}", peers.created, peers.spawn_ok),
    ];
    for check in checks {
        lines.push(format!(
            "{} {} {}",
            if check.ok { "✅" } else { "❌" },
            check.name,
            check.detail
        ));
    }
    lines.push(format!("logs: {}", env.dir.display()));
    lines.push(
        "rule: peers learn each other ONLY via the freenet lobby (no mdns/kad/bootstrap)"
            .to_string(),
    );
    lines.join("\n")
}

// needed helper: informational freenet network status line (no pass/fail)
fn freenet_status(env: &Env, logs: &[PathBuf]) -> String {
    let text = logs
        .first()
        .map_or_else(String::new, |log| read_head(log, 262_144));
    let network = if text.contains("Replacing local gateways with gateways from remote index") {
        "mainnet"
    } else {
        "local/isolated"
    };
    format!(
        "running on: {network} gateway={} mdns=off transport={} token={}",
        gateway_addr(&text),
        env.transport,
        env.token
    )
}

// needed helper: extracts the first gateway ip:port advertised in the log head
fn gateway_addr(text: &str) -> String {
    const MARKER: &str = "gateway gateway=";
    text.lines()
        .find_map(|line| {
            let index = line.find(MARKER)?;
            let rest = line.get(index.checked_add(MARKER.len())?..)?;
            let entry = rest.split_whitespace().next()?;
            entry.split('@').next_back().map(str::to_string)
        })
        .unwrap_or_else(|| "none".to_string())
}

async fn send_clip(clip: Option<&Path>, caption: &str) {
    let Some(clip) = clip else {
        return;
    };
    let clip = clip.to_path_buf();
    let caption = caption.to_string();
    let _ = tokio::task::spawn_blocking(move || send_video_best_effort(&clip, &caption)).await;
}

fn early_report(scenario: &Scenario, detail: String) -> Report {
    Report {
        checks: vec![Check {
            name: "setup".to_string(),
            ok: false,
            detail,
        }],
        caption: format!("{} setup failed", scenario.name),
        logs_dir: PathBuf::new(),
    }
}

// needed helper: spawns one app instance with its action/script/token args
fn spawn_peer(
    scenario: &Scenario,
    env: &Env,
    index: usize,
    username: &str,
    action: Option<&str>,
    windows: &mut Vec<TerminalGuard>,
) -> Result<PathBuf, String> {
    let log = env.dir.join(format!("instance-{username}.log"));
    let mut args = vec![
        "--username".to_string(),
        username.to_string(),
        "--token".to_string(),
        env.token.clone(),
        "--transport".to_string(),
        env.transport.clone(),
    ];
    if let Some(ui) = &scenario.ui {
        args.push("--ui".to_string());
        args.push(ui.clone());
    }
    if let Some(action) = action {
        args.push("--action".to_string());
        args.push(action.to_string());
    }
    if let Some(script) = scenario.scripts.get(index).and_then(Option::as_ref) {
        args.push("--script".to_string());
        args.push(script.clone());
    }
    if scenario.is_room_ping_enabled {
        args.push("--room-ping".to_string());
    }
    let guard =
        super::spawn_app::spawn_app(&env.bin, &args, username, &log, scenario.window_prefix)?;
    windows.push(guard);
    Ok(log)
}

// needed helper: names the peers whose freenet node never read the lobby
fn ring_detail(logs: &[PathBuf], ring_ready: &[bool]) -> String {
    let missing: Vec<String> = ring_ready
        .iter()
        .enumerate()
        .filter(|(_, ready)| !**ready)
        .map(|(index, _)| format!("p{}", index.saturating_add(1)))
        .collect();
    if missing.is_empty() {
        format!("{}/{} peers read the lobby", ring_ready.len(), logs.len())
    } else {
        format!("never read the lobby: {}", missing.join(","))
    }
}

// needed helper: the creator's embedded freenet node seeded a lobby with its room
//
// The deploy ack can land after `REQUEST_TIMEOUT`, so the ack log line alone is a
// false negative; reading back a lobby that already holds our room (the seeded
// `rooms>=1`) is the durable signal that the `Put` landed.
fn creator_published(logs: &[PathBuf]) -> bool {
    let Some(log) = logs.first() else {
        return false;
    };
    log_contains(log, "discovery: lobby contract deployed")
        || ring_ready_rooms(log).is_some_and(|rooms| rooms > 0)
}

// needed helper: the `rooms=<n>` count from the node's `lobby freenet ring ready` line
fn ring_ready_rooms(log: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(log).ok()?;
    text.lines()
        .filter(|line| line.contains("lobby freenet ring ready"))
        .find_map(|line| {
            line.split_whitespace()
                .find_map(|token| token.strip_prefix("rooms="))
                .and_then(|rooms| rooms.parse().ok())
        })
}

// needed helper: peers whose freenet node reported zero ring connections
fn isolated_peers(logs: &[PathBuf]) -> Vec<String> {
    logs.iter()
        .enumerate()
        .filter(|(_, log)| log_contains(log, "Node isolated with zero ring connections"))
        .map(|(index, _)| format!("p{}", index.saturating_add(1)))
        .collect()
}

// needed helper: every peer's latest tick lists the room in the rooms
fn do_all_peers_see_room(logs: &[PathBuf], room: &str) -> bool {
    logs.iter()
        .all(|log| last_tick(log).is_some_and(|tick| tick.rooms.iter().any(|r| r == room)))
}

// needed helper: every peer is in the room with a full mesh
fn are_all_peers_in_room(logs: &[PathBuf], room: &str, peers: usize) -> bool {
    logs.iter().all(|log| {
        last_tick(log).is_some_and(|tick| {
            tick.room.as_deref() == Some(room) && tick.connected == peers.saturating_sub(1)
        })
    })
}

// needed helper: current per-peer tick summary for failure details
fn mesh_snapshot(logs: &[PathBuf], peers: usize) -> String {
    let want = peers.saturating_sub(1);
    logs.iter()
        .enumerate()
        .map(|(index, log)| {
            let tick = last_tick(log);
            let room = tick
                .as_ref()
                .and_then(|t| t.room.clone())
                .unwrap_or_else(|| "none".to_string());
            let connected = tick.as_ref().map_or(0, |t| t.connected);
            format!(
                "p{}:room={room},connected={connected}/{want}",
                index.saturating_add(1)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// needed helper: polls one log until it contains a marker or the timeout elapses
fn did_log_appear_within(log: &Path, needle: &str, timeout_secs: u64) -> bool {
    let start = Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if log_contains(log, needle) {
            return true;
        }
        poke();
        std::thread::sleep(Duration::from_millis(POLL_MS));
    }
    log_contains(log, needle)
}

// needed helper: async polling predicate with screen poke
async fn did_predicate_hold_within(timeout: Duration, mut predicate: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if predicate() {
            return true;
        }
        poke();
        tokio::time::sleep(Duration::from_millis(POLL_MS)).await;
    }
    predicate()
}

// needed helper: unix seconds for unique run/room names
fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_default()
}

// needed helper: room name from the creator's `lobby created room=<r>` marker
fn created_room(log: &Path) -> Option<String> {
    let text = std::fs::read_to_string(log).ok()?;
    text.lines().find_map(|line| {
        let index = line.find(CREATED_MARKER)?;
        let rest = line.get(index.checked_add(CREATED_MARKER.len())?..)?;
        rest.split_whitespace().next().map(str::to_string)
    })
}

// needed helper: last joiner's button press -> every peer has all N-1 links (event-driven)
fn converge_check(logs: &[PathBuf], peers: usize) -> Check {
    let want = format!("connected={}", peers.saturating_sub(1));
    let last_press = logs
        .iter()
        .filter_map(|log| {
            marker_times(log, "lobby intent pressed")
                .first()
                .map(|(ms, _)| *ms)
        })
        .max();
    let meshed: Vec<Option<u64>> = logs
        .iter()
        .map(|log| {
            marker_times(log, "lobby mesh ")
                .into_iter()
                .find(|(ms, line)| {
                    last_press.is_some_and(|press| *ms >= press)
                        && line.split_whitespace().any(|token| token == want)
                })
                .map(|(ms, _)| ms)
        })
        .collect();
    let converge = last_press.and_then(|press| {
        meshed
            .iter()
            .copied()
            .collect::<Option<Vec<u64>>>()
            .and_then(|times| times.into_iter().max())
            .map(|done| done.saturating_sub(press))
    });
    Check {
        name: "converge".to_string(),
        ok: converge.is_some_and(|ms| ms <= CONVERGE_BUDGET_MS),
        detail: converge.map_or_else(
            || "never fully meshed after the last press".to_string(),
            |ms| format!("{ms}ms budget={CONVERGE_BUDGET_MS}ms (last press -> full mesh)"),
        ),
    }
}
