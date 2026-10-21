use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use telegram_bot_helper::send_video_best_effort;

use super::{
    CanvasTick, TerminalGuard, build_game, canvas_ticks, cleanup_stale, finish_record,
    history_write_bytes, last_canvas, last_tick, log_contains, marker_times, poke, read_head,
    require_x11, rings_ready, senders_seen_in_room, speed_clip, start_record, tile, wakeup_screen,
};

pub const DISCOVERY_BUDGET_SECS: u64 = 120;
pub const JOIN_BUDGET_SECS: u64 = 180;
pub const ROOM_MESSAGE_BUDGET_SECS: u64 = 30;
/// libp2p-only join target: last joiner's press -> every peer fully meshed.
pub const CONVERGE_BUDGET_MS: u64 = 500;
/// Starboard only: last scripted stroke -> every peer reporting the same canvas.
pub const CANVAS_BUDGET_SECS: u64 = 30;
/// History only: a holder wrote its room's history into the Kademlia store.
pub const HISTORY_WRITE_BUDGET_SECS: u64 = 120;
/// History only: a holder left the room or quit the game.
pub const HISTORY_LEAVE_BUDGET_SECS: u64 = 420;
/// History only: a late joiner restored the board from the Kademlia history.
/// The mainnet's contract state can take minutes to reach a cold node, so the
/// late joiner's own join is inside this budget.
pub const HISTORY_RESTORE_BUDGET_SECS: u64 = 420;
/// History only: a peer that could not join reported the failure and exited.
pub const HISTORY_FAILURE_BUDGET_SECS: u64 = 420;

const SPAWN_TIMEOUT_SECS: u64 = 180;
const RECORD_SECS: u64 = 600;
const POLL_MS: u64 = 500;
const SPAWN_GAP_MS: u64 = 300;
const CREATED_MARKER: &str = "lobby created room=";
const SCRIPT_DONE_MARKER: &str = "starboard script done";
const JOIN_FAILED_MARKER: &str = "starboard join failed";
const JOIN_FAILED_EXIT_CODE: i32 = 2;

/// What a scenario proves. One variant per shape of run, so the flags that
/// used to spell it out cannot contradict each other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioFlow {
    /// The peers read the lobby and never join a room.
    DiscoveryOnly,
    /// The peers join one room and mesh.
    Join,
    /// The peers join one room and exchange room messages.
    RoomPing,
    /// The starboard peers paint, erase and converge on one canvas.
    Starboard,
    /// The same, plus one peer that joins only after the painting stopped.
    StarboardLateJoin,
    /// S4a: every peer left the room, a late joiner restores the board.
    HistoryRestore,
    /// S4b: every peer quit the game, a later joiner finds the room gone.
    HistoryRoomGone,
    /// S4c: the last holder is killed while a peer is joining.
    HistoryHolderKilled,
}

/// What a peer does with the room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerRole {
    /// Creates the room.
    Creator,
    /// Joins the room.
    Joiner,
    /// Stays in the lobby and never joins a room.
    Watcher,
}

/// When a peer is spawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerSpawn {
    /// With the first batch.
    First,
    /// After the first batch has done its job.
    Late,
}

/// One peer of a scenario. A scripted peer carries its room action in the
/// script, so `--script` and `--action` are never both passed.
#[derive(Debug, Clone)]
pub struct PeerSpec {
    pub role: PeerRole,
    pub spawn: PeerSpawn,
    pub script: Option<String>,
}

/// How long a scenario waits for things.
#[derive(Debug, Clone, Copy)]
pub struct ScenarioBudgets {
    /// Seconds the scenario waits for the lobby row.
    pub discovery: u64,
    /// Seconds the scenario waits for the room to mesh.
    pub join: u64,
    /// Seconds the peer waits for its room to show up in the lobby.
    pub join_timeout: u64,
}

pub struct Scenario {
    pub name: String,
    pub bin: &'static str,
    pub window_prefix: &'static str,
    pub ui: Option<String>,
    pub flow: ScenarioFlow,
    pub peers: Vec<PeerSpec>,
    pub budgets: ScenarioBudgets,
}

impl Scenario {
    #[must_use]
    pub fn discover_only(peers: usize) -> Self {
        Self {
            name: "lobby_room_discovery".to_string(),
            bin: "lobby_room",
            window_prefix: "lobby",
            ui: Some("custom".to_string()),
            flow: ScenarioFlow::DiscoveryOnly,
            peers: roster(peers, PeerRole::Watcher),
            budgets: default_budgets(),
        }
    }

    #[must_use]
    pub fn join(peers: usize) -> Self {
        Self {
            name: "lobby_room_join".to_string(),
            bin: "lobby_room",
            window_prefix: "lobby",
            ui: Some("default".to_string()),
            flow: ScenarioFlow::Join,
            peers: roster(peers, PeerRole::Joiner),
            budgets: default_budgets(),
        }
    }

    #[must_use]
    pub fn room_messaging(peers: usize) -> Self {
        Self {
            name: "room_messaging".to_string(),
            bin: "lobby_room",
            window_prefix: "lobby",
            ui: Some("default".to_string()),
            flow: ScenarioFlow::RoomPing,
            peers: roster(peers, PeerRole::Joiner),
            budgets: default_budgets(),
        }
    }

    /// Three starboard windows: p1 paints, p2 erases some of it, p3 only watches.
    #[must_use]
    pub fn starboard(peers: usize) -> Self {
        Self {
            name: "starboard_e2e".to_string(),
            bin: "starboard_gossip",
            window_prefix: "starboard",
            ui: None,
            flow: ScenarioFlow::Starboard,
            peers: starboard_roster(peers, PeerSpawn::First),
            budgets: default_budgets(),
        }
    }

    /// The same, plus one peer that joins only after the painting stopped.
    #[must_use]
    pub fn starboard_late_join(peers: usize) -> Self {
        Self {
            name: "starboard_e2e".to_string(),
            flow: ScenarioFlow::StarboardLateJoin,
            peers: starboard_roster(peers, PeerSpawn::Late),
            ..Self::starboard(peers)
        }
    }

    /// S4a: two holders paint, leave the room, and a third peer restores the
    /// board from the Kademlia history.
    #[must_use]
    pub fn starboard_history_restore(peers: usize) -> Self {
        Self {
            name: "starboard_history_restore".to_string(),
            bin: "starboard_gossip",
            window_prefix: "starboard",
            ui: None,
            flow: ScenarioFlow::HistoryRestore,
            peers: history_restore_roster(peers),
            budgets: default_budgets(),
        }
    }

    /// S4b: the holders quit the game, a watcher watches the room disappear,
    /// and a later joiner finds the room gone.
    #[must_use]
    pub fn starboard_history_room_gone(peers: usize) -> Self {
        Self {
            name: "starboard_history_room_gone".to_string(),
            bin: "starboard_gossip",
            window_prefix: "starboard",
            ui: None,
            flow: ScenarioFlow::HistoryRoomGone,
            peers: history_room_gone_roster(peers),
            budgets: default_budgets(),
        }
    }

    /// S4c: the only holder is killed while the joiner is joining.
    #[must_use]
    pub fn starboard_history_holder_killed(peers: usize) -> Self {
        Self {
            name: "starboard_history_holder_killed".to_string(),
            bin: "starboard_gossip",
            window_prefix: "starboard",
            ui: None,
            flow: ScenarioFlow::HistoryHolderKilled,
            peers: history_holder_killed_roster(peers),
            budgets: default_budgets(),
        }
    }
}

/// The budgets every scenario starts from.
const fn default_budgets() -> ScenarioBudgets {
    ScenarioBudgets {
        discovery: DISCOVERY_BUDGET_SECS,
        join: JOIN_BUDGET_SECS,
        join_timeout: JOIN_BUDGET_SECS,
    }
}

/// One creator and `peers - 1` peers in the given role.
fn roster(peers: usize, role: PeerRole) -> Vec<PeerSpec> {
    let mut roster = vec![PeerSpec {
        role: PeerRole::Creator,
        spawn: PeerSpawn::First,
        script: None,
    }];
    for _ in 1..peers {
        roster.push(PeerSpec {
            role,
            spawn: PeerSpawn::First,
            script: None,
        });
    }
    roster
}

/// The starboard roster: p1 paints, p2 erases, the rest only watch the board.
fn starboard_roster(peers: usize, late: PeerSpawn) -> Vec<PeerSpec> {
    let mut roster = vec![
        PeerSpec {
            role: PeerRole::Creator,
            spawn: PeerSpawn::First,
            script: Some(format!("create:{{room}}, {}", paint_script())),
        },
        PeerSpec {
            role: PeerRole::Joiner,
            spawn: PeerSpawn::First,
            script: Some(format!("join:{{room}}, {}", erase_script())),
        },
    ];
    for _ in 2..peers {
        roster.push(PeerSpec {
            role: PeerRole::Joiner,
            spawn: PeerSpawn::First,
            script: None,
        });
    }
    if let Some(last) = roster.last_mut() {
        last.spawn = late;
    }
    roster
}

/// Two holders that paint and then leave the room, plus a late joiner.
///
/// Only the first peer paints: the second one would have to *erase* to differ,
/// and an erase published before the gossipsub mesh knows the subscription is
/// lost (the known `NoPeersSubscribedToTopic` window). The point of the scenario
/// is the history restore, not the op path.
fn history_restore_roster(peers: usize) -> Vec<PeerSpec> {
    let holders = peers.saturating_sub(1).max(2);
    let mut roster = vec![
        PeerSpec {
            role: PeerRole::Creator,
            spawn: PeerSpawn::First,
            script: Some(format!(
                "create:{{room}}, {}, wait-secs:45, leave:{{room}}",
                paint_script()
            )),
        },
        PeerSpec {
            role: PeerRole::Joiner,
            spawn: PeerSpawn::First,
            script: Some("join:{room}, wait-secs:45, leave:{room}".to_string()),
        },
    ];
    for _ in 2..holders {
        roster.push(PeerSpec {
            role: PeerRole::Joiner,
            spawn: PeerSpawn::First,
            script: None,
        });
    }
    roster.push(PeerSpec {
        role: PeerRole::Joiner,
        spawn: PeerSpawn::Late,
        script: None,
    });
    roster
}

/// Two holders that paint and then quit the game, a watcher, and a late joiner.
fn history_room_gone_roster(peers: usize) -> Vec<PeerSpec> {
    let mut roster = vec![
        PeerSpec {
            role: PeerRole::Creator,
            spawn: PeerSpawn::First,
            script: Some(format!(
                "create:{{room}}, {}, wait-secs:45, exit",
                paint_script()
            )),
        },
        PeerSpec {
            role: PeerRole::Joiner,
            spawn: PeerSpawn::First,
            script: Some(format!(
                "join:{{room}}, {}, wait-secs:45, exit",
                erase_script()
            )),
        },
        PeerSpec {
            role: PeerRole::Watcher,
            spawn: PeerSpawn::First,
            script: None,
        },
    ];
    for _ in 3..peers.saturating_sub(1) {
        roster.push(PeerSpec {
            role: PeerRole::Watcher,
            spawn: PeerSpawn::First,
            script: None,
        });
    }
    roster.push(PeerSpec {
        role: PeerRole::Joiner,
        spawn: PeerSpawn::Late,
        script: None,
    });
    roster
}

/// One holder that paints, and a joiner that arrives after the holder is killed.
fn history_holder_killed_roster(peers: usize) -> Vec<PeerSpec> {
    let mut roster = vec![PeerSpec {
        role: PeerRole::Creator,
        spawn: PeerSpawn::First,
        script: Some(format!("create:{{room}}, {}", paint_script())),
    }];
    for _ in 1..peers.saturating_sub(1) {
        roster.push(PeerSpec {
            role: PeerRole::Joiner,
            spawn: PeerSpawn::First,
            script: None,
        });
    }
    roster.push(PeerSpec {
        role: PeerRole::Joiner,
        spawn: PeerSpawn::Late,
        script: None,
    });
    roster
}

/// The strokes the painter draws, as the script spells them.
fn paint_script() -> String {
    strokes(6, "paint-from-to")
}

/// The strokes the eraser removes, the same geometry as the first paint strokes.
fn erase_script() -> String {
    strokes(3, "erase-from-to")
}

/// `count` strokes of the scripted session, as one `verb:x.y-x.y` step each.
fn strokes(count: u32, verb: &str) -> String {
    (0..count)
        .map(|index| {
            let step = u16::try_from(index).unwrap_or(0);
            let x = 200_u16.saturating_add(step.saturating_mul(30));
            let y = 300_u16.saturating_add(step.saturating_mul(9));
            format!("{verb}:{x}.{y}-{}.{}", x.saturating_add(60), y)
        })
        .collect::<Vec<String>>()
        .join(", ")
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

struct RunningPeer {
    role: PeerRole,
    spawn: PeerSpawn,
    log: PathBuf,
    guard: TerminalGuard,
}

struct RunningPeers {
    peers: Vec<RunningPeer>,
    room: String,
    created: bool,
    spawn_ok: bool,
    t_create: Instant,
    t_join: Instant,
}

impl RunningPeers {
    /// The peers that are in the room.
    fn room_indices(&self) -> Vec<usize> {
        self.peers
            .iter()
            .enumerate()
            .filter(|(_, peer)| peer.role != PeerRole::Watcher)
            .map(|(index, _)| index)
            .collect()
    }

    /// The peers that only watch the lobby.
    fn watcher_indices(&self) -> Vec<usize> {
        self.peers
            .iter()
            .enumerate()
            .filter(|(_, peer)| peer.role == PeerRole::Watcher)
            .map(|(index, _)| index)
            .collect()
    }

    /// The peers that were spawned after the first batch.
    fn late_indices(&self) -> Vec<usize> {
        self.peers
            .iter()
            .enumerate()
            .filter(|(_, peer)| peer.spawn == PeerSpawn::Late)
            .map(|(index, _)| index)
            .collect()
    }

    fn logs(&self) -> Vec<PathBuf> {
        self.peers.iter().map(|peer| peer.log.clone()).collect()
    }
}

pub async fn run_scenario(scenario: Scenario) -> Report {
    let mut env = match prepare(&scenario) {
        Ok(env) => env,
        Err(detail) => return early_report(&scenario, detail),
    };
    let mut peers = spawn(&scenario, &env);
    let checks = evaluate(&scenario, &env, &mut peers).await;
    let clip = stop_recording(env.recording.take(), &env.raw, &env.clip);
    let windows: Vec<TerminalGuard> = peers
        .peers
        .iter_mut()
        .map(|peer| std::mem::replace(&mut peer.guard, discarded_guard()))
        .collect();
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

fn spawn(scenario: &Scenario, env: &Env) -> RunningPeers {
    let mut peers = Vec::new();
    let mut spawn_ok = true;
    for (index, spec) in scenario.peers.iter().enumerate() {
        if spec.spawn == PeerSpawn::Late {
            continue;
        }
        let username = format!("p{}", index.saturating_add(1));
        let action = room_action(spec.role, &env.room);
        let pushed = push_peer(
            scenario,
            env,
            spec,
            &username,
            action.as_deref(),
            &mut peers,
        );
        if let Err(e) = &pushed {
            eprintln!("{e}");
            spawn_ok = false;
        }
        std::thread::sleep(Duration::from_millis(SPAWN_GAP_MS));
    }
    // the UI picks the room name; read it back from the creator's log
    let room = peers
        .first()
        .map(|peer| peer.log.clone())
        .filter(|log| did_log_appear_within(log, CREATED_MARKER, SPAWN_TIMEOUT_SECS))
        .and_then(|log| created_room(&log))
        .unwrap_or_default();
    let created = !room.is_empty();
    let t_create = Instant::now();
    let t_join = t_create;
    if let Err(e) = tile_peers(&peers) {
        eprintln!("tile: {e}");
    }
    RunningPeers {
        peers,
        room,
        created,
        spawn_ok,
        t_create,
        t_join,
    }
}

/// The `--action` of a peer that is not scripted.
fn room_action(role: PeerRole, room: &str) -> Option<String> {
    match role {
        PeerRole::Creator => Some(format!("create:{room}")),
        PeerRole::Joiner => Some(format!("join:{room}")),
        PeerRole::Watcher => None,
    }
}

fn push_peer(
    scenario: &Scenario,
    env: &Env,
    spec: &PeerSpec,
    username: &str,
    action: Option<&str>,
    peers: &mut Vec<RunningPeer>,
) -> Result<(), String> {
    let log = env.dir.join(format!("instance-{username}.log"));
    let mut args = vec![
        "--username".to_string(),
        username.to_string(),
        "--token".to_string(),
        env.token.clone(),
        "--transport".to_string(),
        env.transport.clone(),
        "--join-timeout-secs".to_string(),
        scenario.budgets.join_timeout.to_string(),
    ];
    if let Some(ui) = &scenario.ui {
        args.push("--ui".to_string());
        args.push(ui.clone());
    }
    match &spec.script {
        Some(script) => {
            args.push("--script".to_string());
            args.push(script.replace("{room}", &env.room));
        }
        None => {
            if let Some(action) = action {
                args.push("--action".to_string());
                args.push(action.to_string());
            }
        }
    }
    let guard =
        super::spawn_app::spawn_app(&env.bin, &args, username, &log, scenario.window_prefix)?;
    peers.push(RunningPeer {
        role: spec.role,
        spawn: spec.spawn,
        log,
        guard,
    });
    Ok(())
}

/// A guard that owns no process, for the peers that already exited.
const fn discarded_guard() -> TerminalGuard {
    TerminalGuard {
        child: None,
        window_title: String::new(),
        log: PathBuf::new(),
    }
}

async fn evaluate(scenario: &Scenario, env: &Env, peers: &mut RunningPeers) -> Vec<Check> {
    // Wait for the lobby row itself, not a transport-connection proxy. `ring ready`
    // is emitted once a node actually reads the lobby, which on mainnet can take
    // tens of seconds; the two checks then separate "a node never read the lobby"
    // (ring) from "lobby read but the row is missing" (discovery).
    let seen = did_predicate_hold_within(Duration::from_secs(scenario.budgets.discovery), || {
        do_all_peers_see_room(&peers.logs(), &peers.room)
    })
    .await;
    let ring_ready = rings_ready(&peers.logs());
    let ring_ok = ring_ready.iter().all(|ready| *ready);
    let discovery_ms = peers.t_create.elapsed().as_millis();
    let discovery_ok = ring_ok
        && peers.created
        && peers.spawn_ok
        && seen
        && peers.t_create.elapsed().as_secs() <= scenario.budgets.discovery;
    let (join_ok, join_ms, join_detail) = evaluate_join(scenario, peers, ring_ok).await;
    let expected = first_batch_size(scenario);
    let spawn_check = Check {
        name: "spawn".to_string(),
        ok: peers.spawn_ok && peers.peers.len() == expected,
        detail: format!("{}/{} instances", peers.peers.len(), expected),
    };
    let published = creator_published(&peers.logs());
    let lobby_check = Check {
        name: "lobby".to_string(),
        ok: peers.created && published,
        detail: format!("created={} published={published}", peers.created),
    };
    let ring_check = Check {
        name: "ring".to_string(),
        ok: ring_ok,
        detail: ring_detail(&peers.logs(), &ring_ready),
    };
    let discovery_check = Check {
        name: "discovery".to_string(),
        ok: discovery_ok,
        detail: format!(
            "{discovery_ms}ms budget={}s created={} seen={seen} isolated=[{}]",
            scenario.budgets.discovery,
            peers.created,
            isolated_peers(&peers.logs()).join(",")
        ),
    };
    let join_check = Check {
        name: "join".to_string(),
        ok: join_ok,
        detail: format!(
            "{join_ms}ms budget={}s {join_detail}",
            scenario.budgets.join
        ),
    };
    let mut checks = vec![
        spawn_check,
        lobby_check,
        ring_check,
        discovery_check,
        join_check,
    ];
    match scenario.flow {
        ScenarioFlow::DiscoveryOnly => {}
        ScenarioFlow::Join => {
            checks.push(converge_check(&peers.logs(), peers.peers.len()));
        }
        ScenarioFlow::RoomPing => {
            checks.push(converge_check(&peers.logs(), peers.peers.len()));
            checks.push(evaluate_room_messages(peers, join_ok).await);
        }
        ScenarioFlow::Starboard => {
            checks.push(converge_check(&peers.logs(), peers.peers.len()));
            checks.push(canvas_check(peers).await);
        }
        ScenarioFlow::StarboardLateJoin => {
            checks.push(converge_check(&peers.logs(), peers.peers.len()));
            checks.push(canvas_check(peers).await);
            checks.push(late_join_check(scenario, env, peers).await);
        }
        ScenarioFlow::HistoryRestore => {
            checks.push(history_restore_check(scenario, env, peers).await);
        }
        ScenarioFlow::HistoryRoomGone => {
            checks.push(history_room_gone_check(scenario, env, peers).await);
        }
        ScenarioFlow::HistoryHolderKilled => {
            checks.push(history_holder_killed_check(scenario, env, peers).await);
        }
    }
    checks
}

// needed helper: peers spawned before the checks run (the late joiner comes later)
fn first_batch_size(scenario: &Scenario) -> usize {
    scenario
        .peers
        .iter()
        .filter(|spec| spec.spawn == PeerSpawn::First)
        .count()
}

// needed helper: positions the peers' windows on the screen
fn tile_peers(peers: &[RunningPeer]) -> Result<(), String> {
    let titles: Vec<&str> = peers
        .iter()
        .map(|peer| peer.guard.window_title.as_str())
        .collect();
    tile(&titles)
}

// needed helper: spawns one late peer and waits for the new mesh
async fn late_join_check(scenario: &Scenario, env: &Env, peers: &mut RunningPeers) -> Check {
    let username = format!("p{}", peers.peers.len().saturating_add(1));
    let Some(spec) = late_spec(scenario) else {
        return Check {
            name: "late_join".to_string(),
            ok: false,
            detail: "the scenario has no late peer".to_string(),
        };
    };
    let start = Instant::now();
    let action = room_action(spec.role, &peers.room);
    let pushed = push_peer(
        scenario,
        env,
        &spec,
        &username,
        action.as_deref(),
        &mut peers.peers,
    );
    if let Err(e) = &pushed {
        eprintln!("{e}");
    }
    let spawned = pushed.is_ok();
    if let Err(e) = tile_peers(&peers.peers) {
        eprintln!("tile: {e}");
    }
    let total = peers.peers.len();
    let meshed = did_predicate_hold_within(Duration::from_secs(scenario.budgets.join), || {
        are_all_peers_in_room(&peers.logs(), &peers.room, total)
    })
    .await;
    let detail = if meshed {
        "full mesh".to_string()
    } else {
        format!("mesh not reached: {}", mesh_snapshot(&peers.logs(), total))
    };
    Check {
        name: "late_join".to_string(),
        ok: spawned && meshed,
        detail: format!(
            "{}ms budget={}s {total} instances {detail}",
            start.elapsed().as_millis(),
            scenario.budgets.join
        ),
    }
}

// needed helper: the scenario's late peer, if it has one
fn late_spec(scenario: &Scenario) -> Option<PeerSpec> {
    scenario
        .peers
        .iter()
        .find(|spec| spec.spawn == PeerSpawn::Late)
        .cloned()
}

// needed helper: every peer's canvas hash agrees after the last scripted stroke
async fn canvas_check(peers: &RunningPeers) -> Check {
    let start = Instant::now();
    let agreed = did_predicate_hold_within(Duration::from_secs(CANVAS_BUDGET_SECS), || {
        do_canvases_agree(&peers.logs())
    })
    .await;
    let snapshot: Vec<String> = peers
        .logs()
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
            marker_times(log, SCRIPT_DONE_MARKER)
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
async fn evaluate_room_messages(peers: &RunningPeers, join_ok: bool) -> Check {
    if !join_ok {
        return Check {
            name: "room_message".to_string(),
            ok: false,
            detail: "skipped: join did not reach full mesh".to_string(),
        };
    }
    let start = Instant::now();
    let want = peers.peers.len().saturating_sub(1);
    let delivered =
        did_predicate_hold_within(Duration::from_secs(ROOM_MESSAGE_BUDGET_SECS), || {
            peers
                .logs()
                .iter()
                .all(|log| senders_seen_in_room(log, &peers.room).len() >= want)
        })
        .await;
    let counts: Vec<String> = peers
        .logs()
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

async fn evaluate_join(
    scenario: &Scenario,
    peers: &RunningPeers,
    ring_ok: bool,
) -> (bool, u128, String) {
    if scenario.flow == ScenarioFlow::DiscoveryOnly {
        return (true, 0, "n/a".to_string());
    }
    if !ring_ok {
        return (
            false,
            peers.t_join.elapsed().as_millis(),
            "skipped: freenet ring never ready".to_string(),
        );
    }
    let logs: Vec<PathBuf> = peers
        .room_indices()
        .iter()
        .filter_map(|index| peers.peers.get(*index))
        .map(|peer| peer.log.clone())
        .collect();
    let mesh = did_predicate_hold_within(Duration::from_secs(scenario.budgets.join), || {
        are_all_peers_in_room(&logs, &peers.room, logs.len())
    })
    .await;
    let budget_ok = peers.t_join.elapsed().as_secs() <= scenario.budgets.join;
    let detail = if mesh {
        "full mesh".to_string()
    } else {
        format!("mesh not reached: {}", mesh_snapshot(&logs, logs.len()))
    };
    (
        mesh && budget_ok,
        peers.t_join.elapsed().as_millis(),
        detail,
    )
}

// needed helper: S4a — the holders wrote their history and left the room, and a
// late joiner restored the same board from the Kademlia history
async fn history_restore_check(scenario: &Scenario, env: &Env, peers: &mut RunningPeers) -> Check {
    let holders = peers.room_indices();
    // One holder is enough: the second one merges the stored history and skips
    // the write when the content is already there.
    let wrote = did_predicate_hold_within(Duration::from_secs(HISTORY_WRITE_BUDGET_SECS), || {
        holders
            .iter()
            .filter_map(|index| peers.peers.get(*index))
            .any(|peer| history_write_bytes(&peer.log).is_some())
    })
    .await;
    let left = did_predicate_hold_within(Duration::from_secs(HISTORY_LEAVE_BUDGET_SECS), || {
        holders
            .iter()
            .filter_map(|index| peers.peers.get(*index))
            .all(|peer| {
                marker_times(&peer.log, "starboard script done leave")
                    .last()
                    .is_some()
            })
    })
    .await;
    let late = spawn_late_peer(scenario, env, peers);
    let expected: Vec<u64> = holders
        .iter()
        .filter_map(|index| peers.peers.get(*index))
        .filter_map(|peer| last_canvas(&peer.log).map(|tick| tick.hash))
        .collect();
    let late_indices = peers.late_indices();
    let restored =
        did_predicate_hold_within(Duration::from_secs(HISTORY_RESTORE_BUDGET_SECS), || {
            let Some(index) = late_indices.first() else {
                return false;
            };
            let Some(peer) = peers.peers.get(*index) else {
                return false;
            };
            let Some(actual) = last_canvas(&peer.log) else {
                return false;
            };
            !expected.is_empty() && expected.iter().all(|hash| *hash == actual.hash)
        })
        .await;
    Check {
        name: "history_restore".to_string(),
        ok: wrote && left && late && restored,
        detail: format!(
            "wrote={wrote} left={left} late_joined={late} restored={restored} expected={expected:?}"
        ),
    }
}

// needed helper: S4b — the holders quit the game and a later joiner reported the
// room as gone and exited with the code. The watcher's view of the room's death
// is evidence, not the requirement: the contract's state reaches the watcher's
// node minutes later, so a watcher can still list a room the contract has
// already dropped.
async fn history_room_gone_check(
    scenario: &Scenario,
    env: &Env,
    peers: &mut RunningPeers,
) -> Check {
    let holders = peers.room_indices();
    let watchers = peers.watcher_indices();
    let quit = did_predicate_hold_within(Duration::from_secs(HISTORY_LEAVE_BUDGET_SECS), || {
        holders
            .iter()
            .filter_map(|index| peers.peers.get(*index))
            .all(|peer| {
                marker_times(&peer.log, "starboard script done exit")
                    .last()
                    .is_some()
            })
    })
    .await;
    let gone = did_predicate_hold_within(Duration::from_secs(HISTORY_LEAVE_BUDGET_SECS), || {
        watchers
            .iter()
            .filter_map(|index| peers.peers.get(*index))
            .all(|peer| !watcher_sees_room(&peer.log, &peers.room))
    })
    .await;
    let late = spawn_late_peer(scenario, env, peers);
    let late_indices = peers.late_indices();
    let failed =
        did_predicate_hold_within(Duration::from_secs(HISTORY_FAILURE_BUDGET_SECS), || {
            late_indices.iter().any(|index| {
                let Some(peer) = peers.peers.get_mut(*index) else {
                    return false;
                };
                log_contains(&peer.log, JOIN_FAILED_MARKER)
                    && exit_code(peer) == Some(JOIN_FAILED_EXIT_CODE)
            })
        })
        .await;
    Check {
        name: "history_room_gone".to_string(),
        ok: quit && late && failed,
        detail: format!(
            "quit={quit} watcher_saw_the_room_gone={gone} late_joined={late} join_failed={failed}"
        ),
    }
}

// needed helper: S4c — the only holder was killed while the joiner was joining,
// and the joiner reported the failure and exited with the code
async fn history_holder_killed_check(
    scenario: &Scenario,
    env: &Env,
    peers: &mut RunningPeers,
) -> Check {
    let Some(spec) = late_spec(scenario) else {
        return Check {
            name: "history_holder_killed".to_string(),
            ok: false,
            detail: "the scenario has no late peer".to_string(),
        };
    };
    let username = format!("p{}", peers.peers.len().saturating_add(1));
    let action = room_action(spec.role, &peers.room);
    let pushed = push_peer(
        scenario,
        env,
        &spec,
        &username,
        action.as_deref(),
        &mut peers.peers,
    );
    if let Err(e) = &pushed {
        eprintln!("{e}");
    }
    let killed = kill_peer(peers.peers.first_mut());
    let late_indices = peers.late_indices();
    let failed =
        did_predicate_hold_within(Duration::from_secs(HISTORY_FAILURE_BUDGET_SECS), || {
            late_indices.iter().any(|index| {
                let Some(peer) = peers.peers.get_mut(*index) else {
                    return false;
                };
                log_contains(&peer.log, JOIN_FAILED_MARKER)
                    && exit_code(peer) == Some(JOIN_FAILED_EXIT_CODE)
            })
        })
        .await;
    Check {
        name: "history_holder_killed".to_string(),
        ok: pushed.is_ok() && killed && failed,
        detail: format!("holder_killed={killed} join_failed={failed}"),
    }
}

// needed helper: spawns the scenario's late peer
fn spawn_late_peer(scenario: &Scenario, env: &Env, peers: &mut RunningPeers) -> bool {
    let Some(spec) = late_spec(scenario) else {
        return false;
    };
    let username = format!("p{}", peers.peers.len().saturating_add(1));
    let action = room_action(spec.role, &peers.room);
    let pushed = push_peer(
        scenario,
        env,
        &spec,
        &username,
        action.as_deref(),
        &mut peers.peers,
    );
    if let Err(e) = &pushed {
        eprintln!("{e}");
    }
    pushed.is_ok()
}

// needed helper: whether the peer already exited, and with which code
fn exit_code(peer: &mut RunningPeer) -> Option<i32> {
    peer.guard
        .child
        .as_mut()
        .and_then(|child| child.try_wait().ok().flatten())
        .map(|status| status.code().unwrap_or(-1))
}

// needed helper: kills a peer, the way a crash would
fn kill_peer(peer: Option<&mut RunningPeer>) -> bool {
    let Some(peer) = peer else {
        return false;
    };
    let Some(child) = peer.guard.child.as_mut() else {
        return false;
    };
    child.kill().is_ok()
}

// needed helper: whether the watcher's lobby still lists the room
fn watcher_sees_room(log: &Path, room: &str) -> bool {
    last_tick(log).is_some_and(|tick| tick.rooms.iter().any(|name| name == room))
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

fn build_caption(scenario: &Scenario, env: &Env, peers: &RunningPeers, checks: &[Check]) -> String {
    let mut lines = vec![
        freenet_status(env, &peers.logs()),
        format!(
            "{} bin={} ui={} room={} transport={} peers={} · build {:.1}s",
            scenario.name,
            scenario.bin,
            scenario.ui.as_deref().unwrap_or("none"),
            peers.room,
            env.transport,
            peers.peers.len(),
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
        || lobby_lists_a_room(log)
}

// needed helper: the creator's own lobby read listed a room (the durable signal
// that its publish landed — the ring-ready line is written once, and it can land
// before the room is in the contract)
fn lobby_lists_a_room(log: &Path) -> bool {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("lobby changed rooms="))
        .any(|line| {
            line.split_whitespace()
                .find_map(|token| token.strip_prefix("rooms="))
                .and_then(|rooms| rooms.parse::<usize>().ok())
                .is_some_and(|rooms| rooms > 0)
        })
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

// needed helper: the current per-peer tick summary for failure details
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

#[cfg(test)]
mod tests {
    use super::{erase_script, paint_script};

    /// The harness writes the script, the example parses it. The shape of every
    /// stroke step is pinned here, and the parser is pinned on the same shape in
    /// `examples/starboard_gossip/script.rs`.
    #[test]
    fn every_scripted_stroke_names_two_cells() {
        for script in [paint_script(), erase_script()] {
            for step in script.split(", ") {
                let (verb, cells) = step.rsplit_once(':').expect("verb:argument");
                assert!(
                    verb.ends_with("-from-to"),
                    "a stroke step is a from-to step: {step}"
                );
                let cells: Vec<&str> = cells.split('-').collect();
                assert_eq!(cells.len(), 2, "a stroke step names two cells: {step}");
                for cell in cells {
                    assert_eq!(cell.split('.').count(), 2, "a cell is x.y: {step}");
                }
            }
        }
    }
}
