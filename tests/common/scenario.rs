use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use telegram_bot_helper::send_video_best_effort;

use super::{
    TerminalGuard, build_game, cleanup_stale, did_rings_become_ready, finish_record, last_tick,
    log_contains, marker_times, poke, read_head, require_x11, speed_clip, start_record, tile,
    wakeup_screen,
};

pub const DISCOVERY_BUDGET_SECS: u64 = 120;
pub const JOIN_BUDGET_SECS: u64 = 180;
/// libp2p-only join target: last joiner's press -> every peer fully meshed.
pub const CONVERGE_BUDGET_MS: u64 = 500;
/// How long a peer's embedded Freenet node is given to join the ring before we
/// stop calling the run a discovery failure (must exceed `Timing.ring`).
pub const RING_BUDGET_SECS: u64 = 45;

const SPAWN_TIMEOUT_SECS: u64 = 180;
const RECORD_SECS: u64 = 600;
const POLL_MS: u64 = 500;
const SPAWN_GAP_MS: u64 = 300;
const CREATED_MARKER: &str = "lobby created room=";

pub struct Scenario {
    pub name: String,
    pub peers: usize,
    pub join: bool,
    pub ui: String,
    pub discovery_budget_secs: u64,
    pub join_budget_secs: u64,
}

impl Scenario {
    #[must_use]
    pub fn discover_only(peers: usize) -> Self {
        Self {
            name: "lobby_room_discovery".to_string(),
            peers,
            join: false,
            ui: "custom".to_string(),
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
        }
    }

    #[must_use]
    pub fn join(peers: usize) -> Self {
        Self {
            name: "lobby_room_join".to_string(),
            peers,
            join: true,
            ui: "default".to_string(),
            discovery_budget_secs: DISCOVERY_BUDGET_SECS,
            join_budget_secs: JOIN_BUDGET_SECS,
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
    ring_ready: Vec<bool>,
    t_create: Instant,
    t_join: Instant,
}

pub async fn run_scenario(scenario: Scenario) -> Report {
    let mut env = match prepare(&scenario) {
        Ok(env) => env,
        Err(detail) => return early_report(&scenario, detail),
    };
    let mut peers = spawn(&scenario, &env);
    let checks = evaluate(&scenario, &peers).await;
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
    let bin = build_game()?;
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
    let first = push_peer(scenario, env, "p1", Some(&create), &mut windows, &mut logs);
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
    for index in 2..=scenario.peers {
        let username = format!("p{index}");
        let action = scenario.join.then(|| format!("join:{room}"));
        if let Err(e) = push_peer(
            scenario,
            env,
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
    let ring_ready = did_rings_become_ready(&logs, Duration::from_secs(RING_BUDGET_SECS));
    Peers {
        windows,
        logs,
        room,
        created,
        spawn_ok,
        ring_ready,
        t_create,
        t_join,
    }
}

fn push_peer(
    scenario: &Scenario,
    env: &Env,
    username: &str,
    action: Option<&str>,
    windows: &mut Vec<TerminalGuard>,
    logs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let log = spawn_peer(
        &env.bin,
        &env.dir,
        username,
        action,
        &env.token,
        &env.transport,
        &scenario.ui,
        windows,
    )?;
    logs.push(log);
    Ok(())
}

async fn evaluate(scenario: &Scenario, peers: &Peers) -> Vec<Check> {
    let ring_ok = peers.ring_ready.iter().all(|ready| *ready);
    let seen = if ring_ok {
        did_predicate_hold_within(Duration::from_secs(scenario.discovery_budget_secs), || {
            do_all_peers_see_room(&peers.logs, &peers.room)
        })
        .await
    } else {
        false
    };
    let discovery_ms = peers.t_create.elapsed().as_millis();
    let discovery_ok = ring_ok
        && peers.created
        && peers.spawn_ok
        && seen
        && peers.t_create.elapsed().as_secs() <= scenario.discovery_budget_secs;
    let (join_ok, join_ms, join_detail) = evaluate_join(scenario, peers, ring_ok).await;
    let spawn_check = Check {
        name: "spawn".to_string(),
        ok: peers.spawn_ok && peers.logs.len() == scenario.peers,
        detail: format!("{}/{} instances", peers.logs.len(), scenario.peers),
    };
    let ring_check = Check {
        name: "ring".to_string(),
        ok: ring_ok,
        detail: ring_detail(peers),
    };
    let discovery_check = Check {
        name: "discovery".to_string(),
        ok: discovery_ok,
        detail: format!(
            "{discovery_ms}ms budget={}s created={} seen={seen}",
            scenario.discovery_budget_secs, peers.created
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
    let mut checks = vec![spawn_check, ring_check, discovery_check, join_check];
    if scenario.join {
        checks.push(converge_check(&peers.logs, scenario.peers));
    }
    checks
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
        are_all_peers_in_room(&peers.logs, &peers.room, scenario.peers)
    })
    .await;
    let budget_ok = peers.t_join.elapsed().as_secs() <= scenario.join_budget_secs;
    let detail = if mesh {
        "full mesh".to_string()
    } else {
        format!(
            "mesh not reached: {}",
            mesh_snapshot(&peers.logs, scenario.peers)
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
            "{} ui={} room={} transport={} peers={} · build {:.1}s",
            scenario.name, scenario.ui, peers.room, env.transport, scenario.peers, env.build_secs
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

// needed helper: spawns one app instance with its action/token args
fn spawn_peer(
    bin: &Path,
    dir: &Path,
    username: &str,
    action: Option<&str>,
    token: &str,
    transport: &str,
    ui: &str,
    windows: &mut Vec<TerminalGuard>,
) -> Result<PathBuf, String> {
    let log = dir.join(format!("instance-{username}.log"));
    let mut args = vec![
        "--username".to_string(),
        username.to_string(),
        "--token".to_string(),
        token.to_string(),
        "--transport".to_string(),
        transport.to_string(),
        "--ui".to_string(),
        ui.to_string(),
    ];
    if let Some(action) = action {
        args.push("--action".to_string());
        args.push(action.to_string());
    }
    let guard = super::spawn_app::spawn_app(bin, &args, username, &log)?;
    windows.push(guard);
    Ok(log)
}

// needed helper: names the peers whose freenet node never joined the ring
fn ring_detail(peers: &Peers) -> String {
    let missing: Vec<String> = peers
        .ring_ready
        .iter()
        .enumerate()
        .filter(|(_, ready)| !**ready)
        .map(|(index, _)| format!("p{}", index.saturating_add(1)))
        .collect();
    if missing.is_empty() {
        format!(
            "{}/{} peers on the freenet ring",
            peers.ring_ready.len(),
            peers.logs.len()
        )
    } else {
        format!("not on the freenet ring: {}", missing.join(","))
    }
}

// needed helper: every peer's latest tick lists the room in the catalogue
fn do_all_peers_see_room(logs: &[PathBuf], room: &str) -> bool {
    logs.iter()
        .all(|log| last_tick(log).is_some_and(|tick| tick.catalogue.iter().any(|r| r == room)))
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
