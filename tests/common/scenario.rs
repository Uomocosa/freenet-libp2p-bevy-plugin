use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use telegram_bot_helper::send_video_best_effort;

use super::{
    TerminalGuard, build_game, cleanup_stale, finish_record, history_write_bytes, last_canvas,
    log_contains, marker_times, read_head, require_x11, speed_clip, start_record, tile,
    wakeup_screen,
};

/// libp2p-only join target: last joiner's press -> every peer fully meshed.
pub const CONVERGE_BUDGET_MS: u64 = 500;

const POLL: Duration = Duration::from_millis(500);
const SPAWN_GAP: Duration = Duration::from_millis(300);
const RECORD_MARGIN_SECS: u64 = 60;
const ROOM: &str = "{room}";
const STEP_DONE_MARKER: &str = "script step done step=";
const SCRIPT_DONE_MARKER: &str = "script done";
const SCRIPT_FAILED_MARKER: &str = "script failed";
const CRASH_MARKER: &str = "script crash-app";

/// When a peer is spawned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerStart {
    /// With the first batch.
    Now,
    /// Once peer `peer` (0-based) logged that its step `step` (1-based) is done.
    AfterStepDone { peer: usize, step: usize },
}

/// One peer of a scenario: its `--script` (with `{room}` for the run's room) and when it starts.
#[derive(Debug, Clone)]
pub struct PeerRun {
    pub script: String,
    pub start: PeerStart,
}

/// A check over every peer's log, run once all the scripts finished. `Ok` carries the detail.
pub type OnFinish = fn(&[PathBuf]) -> Result<String, String>;

/// One e2e run: N scripted peers. It passes when every peer's script ends as written —
/// `exit-app` exits 0, `crash-app` dies, `stay` is still running — and `on_finish` agrees.
pub struct Scenario {
    pub name: String,
    pub bin: &'static str,
    pub window_prefix: &'static str,
    pub ui: Option<String>,
    pub extra_args: Vec<String>,
    pub peers: Vec<PeerRun>,
    pub cap: Duration,
    pub on_finish: Option<OnFinish>,
}

impl Scenario {
    /// The creator stays in its room; every other peer waits for the room in its lobby.
    #[must_use]
    pub fn discover_only(peers: usize) -> Self {
        let mut runs = vec![now(format!("{}, stay", create_and_enter()))];
        for _ in 1..peers {
            runs.push(now("wait-lobby-room:{room}@180s, exit-app".to_string()));
        }
        Self {
            name: "lobby_room_discovery".to_string(),
            ui: Some("custom".to_string()),
            peers: runs,
            cap: Duration::from_mins(5),
            ..Self::lobby()
        }
    }

    /// Every peer ends up in one fully meshed room.
    #[must_use]
    pub fn join(peers: usize) -> Self {
        let meshed = format!("wait-members:{peers}@180s, sleep:5s, exit-app");
        Self {
            name: "lobby_room_join".to_string(),
            ui: Some("default".to_string()),
            peers: room_roster(peers, &meshed),
            cap: Duration::from_mins(7),
            on_finish: Some(did_mesh_converge_within_budget),
            ..Self::lobby()
        }
    }

    /// The meshed room, then room messages from every other member.
    #[must_use]
    pub fn room_messaging(peers: usize) -> Self {
        let others = peers.saturating_sub(1);
        let tail = format!(
            "wait-members:{peers}@180s, wait-room-messages:{others}@60s, sleep:5s, exit-app"
        );
        Self {
            name: "room_messaging".to_string(),
            ui: Some("default".to_string()),
            extra_args: vec!["--room-ping".to_string()],
            peers: room_roster(peers, &tail),
            cap: Duration::from_mins(8),
            on_finish: Some(did_mesh_converge_within_budget),
            ..Self::lobby()
        }
    }

    /// p1 paints, p2 erases some of it, the rest only watch; every canvas ends the same.
    #[must_use]
    pub fn starboard(peers: usize) -> Self {
        let members = format!("wait-members:{peers}@180s");
        let end = "wait-canvas-stable@60s, sleep:10s, exit-app";
        let mut runs = vec![
            now(format!(
                "{}, {members}, {}, {end}",
                create_and_enter(),
                paint_script()
            )),
            now(format!(
                "{}, {members}, wait-canvas-nonempty@120s, {}, {end}",
                join_and_enter(),
                erase_script()
            )),
        ];
        for _ in 2..peers {
            runs.push(now(format!(
                "{}, {members}, wait-canvas-nonempty@120s, {end}",
                join_and_enter()
            )));
        }
        Self {
            name: "starboard_e2e".to_string(),
            peers: runs,
            cap: Duration::from_mins(10),
            on_finish: Some(do_all_canvases_agree),
            ..Self::starboard_base()
        }
    }

    /// The first `peers - 1` paint as in `starboard`, then wait for one more peer that joins
    /// only after the painting stopped and must end with the same canvas.
    #[must_use]
    pub fn starboard_late_join(peers: usize) -> Self {
        let first = peers.saturating_sub(1);
        let members = format!("wait-members:{first}@180s");
        let end = format!("wait-canvas-stable@60s, wait-members:{peers}@420s, sleep:10s, exit-app");
        let painter = format!(
            "{}, {members}, {}, {end}",
            create_and_enter(),
            paint_script()
        );
        let mut runs = vec![now(painter.clone())];
        for _ in 1..first {
            runs.push(now(format!(
                "{}, {members}, wait-canvas-nonempty@120s, {end}",
                join_and_enter()
            )));
        }
        runs.push(PeerRun {
            script: format!(
                "{}, wait-canvas-nonempty@120s, wait-canvas-stable@60s, sleep:10s, exit-app",
                late_join_and_enter()
            ),
            start: after(0, last_step_matching(&painter, "paint-from-to")),
        });
        Self {
            name: "starboard_late_join".to_string(),
            peers: runs,
            cap: Duration::from_mins(15),
            on_finish: Some(do_all_canvases_agree),
            ..Self::starboard_base()
        }
    }

    /// S4a: the holders paint, stay while the history is written, and leave the room; a late
    /// joiner restores the board from the Kademlia history.
    #[must_use]
    pub fn starboard_history_restore(peers: usize) -> Self {
        let holders = peers.saturating_sub(1).max(2);
        let members = format!("wait-members:{holders}@180s");
        let leave = "sleep:45s, leave:{room}, wait-left:{room}@30s, exit-app";
        let mut runs = vec![now(format!(
            "{}, {members}, {}, wait-canvas-stable@60s, {leave}",
            create_and_enter(),
            paint_script()
        ))];
        let holder = format!(
            "{}, {members}, wait-canvas-nonempty@120s, {leave}",
            join_and_enter()
        );
        for _ in 1..holders {
            runs.push(now(holder.clone()));
        }
        runs.push(PeerRun {
            script: format!(
                "{}, wait-canvas-nonempty@120s, wait-canvas-stable@60s, exit-app",
                late_join_and_enter()
            ),
            start: after(
                holders.saturating_sub(1),
                last_step_matching(&holder, "wait-left"),
            ),
        });
        Self {
            name: "starboard_history_restore".to_string(),
            peers: runs,
            cap: Duration::from_mins(20),
            on_finish: Some(was_history_written_and_restored),
            ..Self::starboard_base()
        }
    }

    /// S4b: the holders quit the game, the watchers stay in the lobby, and a later joiner
    /// finds the room gone.
    #[must_use]
    pub fn starboard_history_room_gone(peers: usize) -> Self {
        let members = "wait-members:2@180s";
        let quit = "wait-canvas-stable@60s, sleep:45s, exit-app";
        let painter = format!(
            "{}, {members}, {}, {quit}",
            create_and_enter(),
            paint_script()
        );
        let mut runs = vec![
            now(painter.clone()),
            now(format!(
                "{}, {members}, wait-canvas-nonempty@120s, {}, {quit}",
                join_and_enter(),
                erase_script()
            )),
        ];
        for _ in 2..peers.saturating_sub(1) {
            runs.push(now("wait-lobby-room:{room}@180s, stay".to_string()));
        }
        runs.push(PeerRun {
            script: "join:{room}, wait-join-failed:{room}@420s, exit-app".to_string(),
            start: after(0, step_count(&painter)),
        });
        Self {
            name: "starboard_history_room_gone".to_string(),
            peers: runs,
            cap: Duration::from_mins(20),
            ..Self::starboard_base()
        }
    }

    /// S4c: the only holder crashes while the joiner is joining.
    #[must_use]
    pub fn starboard_history_holder_killed(peers: usize) -> Self {
        let holder = format!(
            "{}, {}, wait-canvas-stable@60s, sleep:5s, crash-app",
            create_and_enter(),
            paint_script()
        );
        let mut runs = vec![now(holder.clone())];
        for _ in 1..peers.saturating_sub(1) {
            runs.push(now("wait-lobby-room:{room}@180s, stay".to_string()));
        }
        runs.push(PeerRun {
            script: "join:{room}, wait-join-failed:{room}@420s, exit-app".to_string(),
            start: after(0, last_step_matching(&holder, "wait-canvas-stable")),
        });
        Self {
            name: "starboard_history_holder_killed".to_string(),
            peers: runs,
            cap: Duration::from_mins(15),
            ..Self::starboard_base()
        }
    }

    // needed helper: the fields every lobby_room scenario shares
    fn lobby() -> Self {
        Self {
            name: String::new(),
            bin: "lobby_room",
            window_prefix: "lobby",
            ui: None,
            extra_args: Vec::new(),
            peers: Vec::new(),
            cap: Duration::from_mins(5),
            on_finish: None,
        }
    }

    // needed helper: the fields every starboard_gossip scenario shares
    fn starboard_base() -> Self {
        Self {
            bin: "starboard_gossip",
            window_prefix: "starboard",
            ..Self::lobby()
        }
    }
}

/// A peer of the first batch.
const fn now(script: String) -> PeerRun {
    PeerRun {
        script,
        start: PeerStart::Now,
    }
}

/// When a late peer starts: once `peer` finished its 1-based step `step`.
const fn after(peer: usize, step: usize) -> PeerStart {
    PeerStart::AfterStepDone { peer, step }
}

/// The creator's opening: create the room and be in it.
fn create_and_enter() -> String {
    "create:{room}, wait-join-succeeded:{room}@180s".to_string()
}

/// A first-batch joiner's opening: see the room, join it, be in it.
fn join_and_enter() -> String {
    "wait-lobby-room:{room}@180s, join:{room}, wait-join-succeeded:{room}@180s".to_string()
}

/// A late joiner's opening: a cold node can take minutes to read the lobby.
fn late_join_and_enter() -> String {
    "wait-lobby-room:{room}@420s, join:{room}, wait-join-succeeded:{room}@420s".to_string()
}

/// One creator and `peers - 1` joiners, all ending with `tail`.
fn room_roster(peers: usize, tail: &str) -> Vec<PeerRun> {
    let mut runs = vec![now(format!("{}, {tail}", create_and_enter()))];
    for _ in 1..peers {
        runs.push(now(format!("{}, {tail}", join_and_enter())));
    }
    runs
}

/// The steps of a script, trimmed.
fn script_steps(script: &str) -> Vec<&str> {
    script.split(',').map(str::trim).collect()
}

/// How many steps a script has.
fn step_count(script: &str) -> usize {
    script_steps(script).len()
}

/// The 1-based position of the last step starting with `verb`, or of the last step.
fn last_step_matching(script: &str, verb: &str) -> usize {
    let steps = script_steps(script);
    steps
        .iter()
        .rposition(|step| step.starts_with(verb))
        .map_or(steps.len(), |index| index.saturating_add(1))
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

/// How a peer's script ends, which decides what "done" means for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScriptEnding {
    ExitApp,
    CrashApp,
    Stay,
}

fn script_ending(script: &str) -> ScriptEnding {
    match script_steps(script).last().copied() {
        Some("crash-app") => ScriptEnding::CrashApp,
        Some("stay") => ScriptEnding::Stay,
        _ => ScriptEnding::ExitApp,
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

/// How a peer's process ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerExit {
    Code(i32),
    Signal,
}

/// A peer of the run: its log, its process once it was spawned, and how it ended.
struct RunningPeer {
    log: PathBuf,
    guard: Option<TerminalGuard>,
    exit: Option<PeerExit>,
}

impl RunningPeer {
    // needed helper: records the exit code the first time the process is seen gone
    fn poll_exit(&mut self) {
        if self.exit.is_some() {
            return;
        }
        let Some(child) = self.guard.as_mut().and_then(|guard| guard.child.as_mut()) else {
            return;
        };
        if let Ok(Some(status)) = child.try_wait() {
            self.exit = Some(status.code().map_or(PeerExit::Signal, PeerExit::Code));
        }
    }
}

pub async fn run_scenario(scenario: Scenario) -> Report {
    let mut env = match prepare(&scenario) {
        Ok(env) => env,
        Err(detail) => return early_report(&scenario, detail),
    };
    let mut peers: Vec<RunningPeer> = (0..scenario.peers.len())
        .map(|index| RunningPeer {
            log: env
                .dir
                .join(format!("instance-p{}.log", index.saturating_add(1))),
            guard: None,
            exit: None,
        })
        .collect();
    drive(&scenario, &env, &mut peers).await;
    let mut checks: Vec<Check> = scenario
        .peers
        .iter()
        .zip(peers.iter())
        .enumerate()
        .map(|(index, (run, peer))| peer_check(index, run, peer))
        .collect();
    if let Some(on_finish) = scenario.on_finish {
        let logs: Vec<PathBuf> = peers.iter().map(|peer| peer.log.clone()).collect();
        let result = on_finish(&logs);
        checks.push(Check {
            name: "on_finish".to_string(),
            ok: result.is_ok(),
            detail: result.unwrap_or_else(|detail| detail),
        });
    }
    let clip = stop_recording(env.recording.take(), &env.raw, &env.clip);
    let windows: Vec<TerminalGuard> = peers
        .iter_mut()
        .filter_map(|peer| peer.guard.take())
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
    let recording = start_record(
        scenario.cap.as_secs().saturating_add(RECORD_MARGIN_SECS),
        &raw,
    );
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

/// Spawns the peers as their starts come due and waits until every script ended, one
/// failed, or the cap is reached.
async fn drive(scenario: &Scenario, env: &Env, peers: &mut [RunningPeer]) {
    let started = Instant::now();
    loop {
        let mut did_spawn = false;
        for index in 0..peers.len() {
            if peers.get(index).is_some_and(|peer| peer.guard.is_some()) {
                continue;
            }
            let Some(run) = scenario.peers.get(index) else {
                continue;
            };
            if !is_start_due(run.start, peers) {
                continue;
            }
            match spawn_peer(scenario, env, index, run) {
                Ok(guard) => {
                    if let Some(peer) = peers.get_mut(index) {
                        peer.guard = Some(guard);
                    }
                    did_spawn = true;
                }
                Err(e) => eprintln!("spawn p{}: {e}", index.saturating_add(1)),
            }
            tokio::time::sleep(SPAWN_GAP).await;
        }
        if did_spawn {
            tile_peers(peers);
        }
        peers.iter_mut().for_each(RunningPeer::poll_exit);
        if has_any_script_failed(scenario, peers)
            || have_all_scripts_ended(scenario, peers)
            || started.elapsed() >= scenario.cap
        {
            return;
        }
        tokio::time::sleep(POLL).await;
    }
}

// needed helper: whether a peer's start condition holds now
fn is_start_due(start: PeerStart, peers: &[RunningPeer]) -> bool {
    match start {
        PeerStart::Now => true,
        PeerStart::AfterStepDone { peer, step } => peers.get(peer).is_some_and(|trigger| {
            log_contains(&trigger.log, &format!("{STEP_DONE_MARKER}{step}/"))
        }),
    }
}

fn spawn_peer(
    scenario: &Scenario,
    env: &Env,
    index: usize,
    run: &PeerRun,
) -> Result<TerminalGuard, String> {
    let username = format!("p{}", index.saturating_add(1));
    let log = env.dir.join(format!("instance-{username}.log"));
    let mut args = vec![
        "--username".to_string(),
        username.clone(),
        "--token".to_string(),
        env.token.clone(),
        "--transport".to_string(),
        env.transport.clone(),
        "--script".to_string(),
        run.script.replace(ROOM, &env.room),
    ];
    if let Some(ui) = &scenario.ui {
        args.push("--ui".to_string());
        args.push(ui.clone());
    }
    args.extend(scenario.extra_args.iter().cloned());
    super::spawn_app::spawn_app(&env.bin, &args, &username, &log, scenario.window_prefix)
}

fn has_any_script_failed(scenario: &Scenario, peers: &[RunningPeer]) -> bool {
    scenario.peers.iter().zip(peers).any(|(run, peer)| {
        log_contains(&peer.log, SCRIPT_FAILED_MARKER)
            || (script_ending(&run.script) != ScriptEnding::CrashApp
                && peer.exit.is_some_and(|exit| exit != PeerExit::Code(0)))
    })
}

fn have_all_scripts_ended(scenario: &Scenario, peers: &[RunningPeer]) -> bool {
    scenario
        .peers
        .iter()
        .zip(peers)
        .all(|(run, peer)| match script_ending(&run.script) {
            ScriptEnding::Stay => true,
            ScriptEnding::ExitApp | ScriptEnding::CrashApp => peer.exit.is_some(),
        })
}

/// Whether one peer's script ended the way it was written to.
fn peer_check(index: usize, run: &PeerRun, peer: &RunningPeer) -> Check {
    let name = format!("p{}", index.saturating_add(1));
    let progress = script_progress(&peer.log, step_count(&run.script));
    if peer.guard.is_none() && peer.exit.is_none() {
        return Check {
            name,
            ok: false,
            detail: format!("never started ({:?})", run.start),
        };
    }
    if let Some((_, line)) = marker_times(&peer.log, SCRIPT_FAILED_MARKER).last() {
        return Check {
            name,
            ok: false,
            detail: after_marker(line, SCRIPT_FAILED_MARKER),
        };
    }
    let (ok, outcome) = match script_ending(&run.script) {
        ScriptEnding::ExitApp => (
            peer.exit == Some(PeerExit::Code(0)) && log_contains(&peer.log, SCRIPT_DONE_MARKER),
            format!("exit={:?}", peer.exit),
        ),
        ScriptEnding::CrashApp => (
            peer.exit.is_some() && log_contains(&peer.log, CRASH_MARKER),
            format!("crashed={}", peer.exit.is_some()),
        ),
        ScriptEnding::Stay => {
            let before_stay = step_count(&run.script).saturating_sub(1);
            (
                peer.exit.is_none()
                    && (before_stay == 0
                        || log_contains(&peer.log, &format!("{STEP_DONE_MARKER}{before_stay}/"))),
                format!("still running={}", peer.exit.is_none()),
            )
        }
    };
    Check {
        name,
        ok,
        detail: format!("{outcome} {progress}"),
    }
}

// needed helper: the last step a peer finished, as `done k/n <step>`
fn script_progress(log: &Path, steps: usize) -> String {
    marker_times(log, STEP_DONE_MARKER).last().map_or_else(
        || format!("done 0/{steps}"),
        |(_, line)| format!("done {}", after_marker(line, STEP_DONE_MARKER)),
    )
}

/// The rest of a log line after `marker`, trimmed.
fn after_marker(line: &str, marker: &str) -> String {
    line.find(marker)
        .and_then(|index| line.get(index.saturating_add(marker.len())..))
        .unwrap_or(line)
        .trim()
        .to_string()
}

// needed helper: positions the live peers' windows on the screen
fn tile_peers(peers: &[RunningPeer]) {
    let titles: Vec<&str> = peers
        .iter()
        .filter(|peer| peer.exit.is_none())
        .filter_map(|peer| peer.guard.as_ref())
        .map(|guard| guard.window_title.as_str())
        .collect();
    if let Err(e) = tile(&titles) {
        eprintln!("tile: {e}");
    }
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

fn build_caption(
    scenario: &Scenario,
    env: &Env,
    peers: &[RunningPeer],
    checks: &[Check],
) -> String {
    let logs: Vec<PathBuf> = peers.iter().map(|peer| peer.log.clone()).collect();
    let mut lines = vec![
        freenet_status(env, &logs),
        format!(
            "{} bin={} ui={} room={} transport={} peers={} · build {:.1}s",
            scenario.name,
            scenario.bin,
            scenario.ui.as_deref().unwrap_or("none"),
            env.room,
            env.transport,
            peers.len(),
            env.build_secs
        ),
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

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// `on_finish`: last joiner's button press -> every peer has all N-1 links (event-driven).
fn did_mesh_converge_within_budget(logs: &[PathBuf]) -> Result<String, String> {
    let want = format!("connected={}", logs.len().saturating_sub(1));
    let last_press = logs
        .iter()
        .filter_map(|log| {
            marker_times(log, "lobby intent pressed")
                .first()
                .map(|(ms, _)| *ms)
        })
        .max()
        .ok_or_else(|| "no peer pressed a room button".to_string())?;
    let meshed: Option<Vec<u64>> = logs
        .iter()
        .map(|log| {
            marker_times(log, "lobby mesh ")
                .into_iter()
                .find(|(ms, line)| {
                    *ms >= last_press && line.split_whitespace().any(|token| token == want)
                })
                .map(|(ms, _)| ms)
        })
        .collect();
    let converge_ms = meshed
        .and_then(|times| times.into_iter().max())
        .map(|done| done.saturating_sub(last_press))
        .ok_or_else(|| "never fully meshed after the last press".to_string())?;
    let detail =
        format!("converge {converge_ms}ms budget={CONVERGE_BUDGET_MS}ms (last press -> full mesh)");
    if converge_ms <= CONVERGE_BUDGET_MS {
        Ok(detail)
    } else {
        Err(detail)
    }
}

/// `on_finish`: every peer's last logged canvas has the same hash, and it is not empty.
fn do_all_canvases_agree(logs: &[PathBuf]) -> Result<String, String> {
    let canvases: Vec<Option<(u64, usize)>> = logs
        .iter()
        .map(|log| last_canvas(log).map(|tick| (tick.hash, tick.tiles)))
        .collect();
    let detail = format!("last canvas (hash, tiles) per peer: {canvases:?}");
    let first = canvases.first().copied().flatten();
    let does_every_peer_agree = first.is_some_and(|(hash, tiles)| {
        tiles > 0
            && canvases
                .iter()
                .all(|canvas| canvas.is_some_and(|(other, _)| other == hash))
    });
    if does_every_peer_agree {
        Ok(detail)
    } else {
        Err(detail)
    }
}

/// `on_finish` (S4a): a holder wrote the room history, and the late joiner (the last peer)
/// ended on the painter's canvas.
fn was_history_written_and_restored(logs: &[PathBuf]) -> Result<String, String> {
    let written = logs.iter().find_map(|log| history_write_bytes(log));
    let painted = logs
        .first()
        .and_then(|log| last_canvas(log))
        .map(|tick| tick.hash);
    let restored = logs
        .last()
        .and_then(|log| last_canvas(log))
        .map(|tick| tick.hash);
    let detail = format!(
        "history_snapshot_bytes={written:?} painter_hash={painted:?} late_joiner_hash={restored:?}"
    );
    if written.is_some() && painted.is_some() && painted == restored {
        Ok(detail)
    } else {
        Err(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Scenario, ScriptEnding, erase_script, last_step_matching, paint_script, script_ending,
    };

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

    #[test]
    fn every_wait_in_every_scenario_carries_a_deadline() {
        let scenarios = [
            Scenario::discover_only(3),
            Scenario::join(5),
            Scenario::room_messaging(3),
            Scenario::starboard(3),
            Scenario::starboard_late_join(4),
            Scenario::starboard_history_restore(3),
            Scenario::starboard_history_room_gone(4),
            Scenario::starboard_history_holder_killed(2),
        ];
        for scenario in scenarios {
            for run in &scenario.peers {
                for step in run.script.split(',').map(str::trim) {
                    assert_eq!(
                        step.starts_with("wait-"),
                        step.contains('@'),
                        "{}: wait steps and only wait steps carry @Ns: {step}",
                        scenario.name
                    );
                }
            }
        }
    }

    #[test]
    fn a_script_ends_as_its_last_step_says() {
        assert_eq!(script_ending("join:r, stay"), ScriptEnding::Stay);
        assert_eq!(script_ending("join:r, crash-app"), ScriptEnding::CrashApp);
        assert_eq!(script_ending("join:r, exit-app"), ScriptEnding::ExitApp);
        assert_eq!(last_step_matching("a, paint-1, paint-2, b", "paint"), 3);
    }
}
