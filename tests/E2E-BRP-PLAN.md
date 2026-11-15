# E2E BRP plan — drive the real apps from outside, the same way on every OS

## 0. Goal

Tests launch the real game, one process per peer, and drive it through the
**Bevy Remote Protocol (BRP)** on localhost: synthetic pointer and keyboard in,
typed status out. No xdotool, no Xvfb for input, no step-script strings. The
examples stay plain examples; everything test-only lives in `tests/e2e/`.

Windows comes later. The design must not block it, but nothing here is tested on it yet.

## 1. Decisions (proposed)

| # | Topic | Decision |
|---|---|---|
| B1 | Driver | `bevy_remote` HTTP JSON-RPC, bound to `127.0.0.1` only. Never exposed on a network: it can mutate the whole `World`. |
| B2 | Ownership | **Each crate adds only what it owns** (RATIONALE §16). Core: `E2EDriverPlugin` and the `E2EMethods` trait. Lobby-room: `LobbyE2EMethods`. Starboard: `StarboardE2EMethods` in `tests/e2e/starboard/common/`. |
| B3 | Clean examples | `examples/starboard` gains a `lib.rs` that exports its game plugin and types; `main.rs` stays tiny and has no `e2e` code. `tests/e2e/starboard/` owns a `starboard_e2e` binary = the same app + driver + fake network. Supersedes L8 (`FakeNetworkPlugin::from_args` leaves the example's `main`). |
| B4 | Cursor | The cursor **moves, never teleports**. `move_cursor(target, speed)` glides from the current position; `click` moves first, then presses. The driver **never touches `Window::cursor_position`**: `bevy_winit` 0.20.1 (`system.rs:426`) syncs a changed value to the OS, which would warp the real mouse. Instead it writes what winit writes: each `CursorMoved` / `MouseButtonInput` / `KeyboardInput` plus its `WindowEvent` twin (`bevy_picking` reads `WindowEvent`). |
| B4b | Game reads picking | `track_cursor` and `track_pointer` read the mouse `PointerLocation` (`bevy_picking`) instead of `Window::cursor_position()`. A game change, not a test hook: same behaviour with a real mouse, and synthetic input reaches it without moving the OS cursor. |
| B5 | Curves | Core owns `PointerSpeed` (logical px/s, time-based, frame-rate independent) and `PointerPath { button, points: Vec<Vec2>, speed }` (window-space polyline). No `Curve` newtype: a polyline has no invariant (LELE026). Starboard's `Stroke` (board cells) lives in tests and becomes `points` through `starboard/board_to_window`. |
| B6 | Waits | Waits are harness-side polling of typed status (`lobby/status`, `starboard/status`). The app side only tracks what a poll cannot see, e.g. `CanvasHashWatch` (stable since). |
| B7 | Orchestrator | **Stays, slimmer.** It owns the local peers (launch, BRP handle, kill), the fake-network router (`PeerMessageRouter`) for no-network runs, and the `CheckpointBoard`. Script parsing and running are deleted. |
| B8 | Cross-machine | Every machine runs the same test program, launches **only its own peers** (`E2E_MACHINE`, `E2E_PLACEMENT`, as L12), and drives them over its own localhost BRP. Machines share checkpoints and final canvas hashes through the `CheckpointBoard` (L13). Peers talk to each other over the real network only. |
| B9 | Placement | `E2E_PLACEMENT=all-local` runs every machine's part on one machine, concurrently, with `LocalCheckpointBoard`. The same test file serves both. |
| B10 | Video | Unchanged: ffmpeg records the real window (x11grab on Linux, gdigrab later). |

## 2. Layout

```text
crates/freenet-libp2p-bevy-core/src/testing/
  __traits__/e2e_methods.rs          trait E2EMethods
  e2e_driver_plugin.rs               BRP server + generic methods
  __basic__/structs.rs               Curve, PointerSpeed, PointerPath
  synthetic_input_queue.rs           queue + glide
  __bevy_systems__/drain_synthetic_input.rs
crates/freenet-libp2p-bevy-lobby-room/src/testing/
  lobby_e2e_methods.rs               lobby/status
examples/starboard/src/
  lib.rs                             pub mod starboard; pub use StarboardApp
  main.rs                            ~10 lines, no e2e
tests/e2e/
  driver/                            Peer, BrpClient, Wait, poll_until
  starboard/common/                  StarboardE2EMethods, CanvasHashWatch, board_to_window
  starboard/bin/starboard_e2e.rs     the wrapper binary
  starboard/starboard_peer.rs        paint, erase, create_room, …
  tests/starboard/*.rs               scenarios
```

## 3. Previews

### B4: the cursor glides

```rust
// crates/freenet-libp2p-bevy-core/src/testing/synthetic_input_queue.rs
#[derive(Resource, Debug, Default)]
pub struct SyntheticInputQueue {
    pub cursor: Option<Vec2>,             // where the synthetic cursor is now
    pub pending: VecDeque<SyntheticInput>,
}

impl SyntheticInputQueue {
    /// Queue a glide from the current position to `target`; nothing jumps.
    pub fn move_cursor(&mut self, target: Vec2, speed: testing::PointerSpeed) {
        let from = self.last_queued_cursor().unwrap_or(target);
        self.pending.push_back(SyntheticInput::Glide { from, to: target, speed });
    }

    pub fn follow(&mut self, path: &testing::PointerPath) {
        let Some(start) = path.curve.first() else { return };
        self.move_cursor(start, path.speed);                  // travel to the stroke start
        self.pending.push_back(SyntheticInput::Button(path.button, ButtonState::Pressed));
        path.curve.segments().for_each(|(from, to)| {
            self.pending.push_back(SyntheticInput::Glide { from, to, speed: path.speed });
        });
        self.pending.push_back(SyntheticInput::Button(path.button, ButtonState::Released));
    }
}
```

`drain_synthetic_input` advances the front `Glide` by `speed * time.delta_secs()` each
frame, writes that frame's position, and pops the glide when it reaches `to`. A slow frame
means a longer step, not a slower stroke. Per frame it writes exactly what winit would:

```rust
fn write_cursor_moved(window: Entity, position: Vec2, out: &mut SyntheticWriters) {
    let event = CursorMoved { window, position, delta: None };
    out.window_events.write(WindowEvent::CursorMoved(event.clone()));   // bevy_picking reads this
    out.cursor_moved.write(event);                                       // anything reading CursorMoved
    // never: window.set_cursor_position(..) — bevy_winit would warp the real OS cursor
}
```

```rust
// examples/starboard/src/starboard/input/bevy_systems/track_cursor.rs (B4b)
pub fn track_cursor(
    pointers: Query<(&PointerId, &PointerLocation)>,
    view: Res<starboard::render::CanvasView>,
    mut cursor: ResMut<starboard::input::CursorCell>,
    mut point: ResMut<starboard::input::CursorPoint>,
) {
    let screen = pointers
        .iter()
        .find_map(|(id, location)| id.is_mouse().then(|| location.location()).flatten())
        .map(|location| location.position);
    // … unchanged: cell_under_cursor / sub_cell_under_cursor
}
```

### B5: `Curve` in core, `Stroke` in starboard's tests

```rust
// crates/freenet-libp2p-bevy-core/src/testing/__basic__/structs.rs
/// A window-space polyline (logical pixels).
#[derive(Debug, Clone, Serialize, Deserialize, Deref, From)]
pub struct Curve(pub Vec<Vec2>);

/// Logical pixels per second.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Deref, From)]
pub struct PointerSpeed(pub f32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerPath {
    pub button: MouseButton,
    pub curve: Curve,
    pub speed: PointerSpeed,
}
```

```rust
// tests/e2e/starboard/stroke.rs
/// Board cells, independent of window size.
#[derive(Debug, Clone, Copy)]
pub struct Stroke(pub &'static [(f32, f32)]);

impl StarboardPeer {
    pub async fn paint(&self, stroke: Stroke) -> anyhow::Result<()> {
        self.follow(MouseButton::Left, stroke, HAND_SPEED).await
    }

    async fn follow(&self, button: MouseButton, stroke: Stroke, speed: PointerSpeed) -> anyhow::Result<()> {
        let curve: Curve = self.0.call("starboard/board_to_window", json!({ "cells": stroke.0 })).await?;
        self.0.follow(&PointerPath { button, curve, speed }).await   // returns when input is idle
    }
}
```

The core `Curve` is a plain polyline. Starboard already smooths strokes with its centripetal
spline, so a smoothed `Curve` would be tested twice. Add `Curve::arc` and the like only when
a scenario needs one.

### B3: the example stays clean; tests own a wrapper binary

```rust
// examples/starboard/src/main.rs
fn main() -> AppExit {
    let args = starboard::Args::parse();
    App::new().add_plugins(starboard::StarboardApp::from(args)).run()
}
```

```rust
// tests/e2e/starboard/bin/starboard_e2e.rs
fn main() -> AppExit {
    let args = starboard_e2e::Args::parse();             // game args + --e2e-port + --network
    App::new()
        .add_plugins(testing::FakeNetworkPlugin::from_args(&args.network))   // before the game
        .add_plugins(starboard::StarboardApp::from(args.game))
        .add_plugins(
            testing::E2EDriverPlugin::new(args.e2e_port)
                .with(lobby_room::testing::LobbyE2EMethods)
                .with(starboard_e2e::StarboardE2EMethods),
        )
        // several peers share one display: unfocused windows must keep updating,
        // or strokes slow down and BRP calls (answered once per frame) hang
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::Continuous,
        })
        .run()
}
```

**Trade-off:** tests exercise `StarboardApp`, not the literal `main.rs`. That gap stays
small as long as `main` is two lines. A `test_usage` that builds both apps with
`MinimalPlugins` checks they agree on the plugin set.

### B8: one test, several machines

```rust
// tests/e2e/tests/starboard/cross_machine_converge.rs
const ROOM_CREATED: Checkpoint = Checkpoint("room-created");
const PAINTED: Checkpoint = Checkpoint("painted");

#[e2e_test(game = "starboard", machines = ["linux", "windows"])]
async fn cross_machine_converge(t: &E2E) -> anyhow::Result<()> {
    let room = t.room_name();                             // same on every machine (run id)

    let linux = t.on("linux", async |m| {
        let alice = m.launch_starboard_peer("alice").await?;
        alice.create_room(&room).await?;
        m.board.mark(ROOM_CREATED).await?;
        alice.wait_until_room_has_members(2).within(m.scaled(420)).await?;
        alice.paint(ALICE_LINE).await?;
        m.board.mark(PAINTED).await?;
        m.publish_final_canvas(&alice, m.scaled(5)).await  // waits stable, posts hash
    });

    let windows = t.on("windows", async |m| {
        m.board.wait(ROOM_CREATED).within(m.scaled(300)).await?;
        let bob = m.launch_starboard_peer("bob").await?;
        bob.join_room(&room).await?;
        bob.paint(BOB_LINE).await?;
        m.board.wait(PAINTED).await?;
        m.publish_final_canvas(&bob, m.scaled(5)).await
    });

    tokio::try_join!(linux, windows)?;                    // `on` is a no-op off its machine
    t.expect_published_canvases_match(["alice", "bob"]).within(t.scaled(120)).await
}
```

- On the Linux runner, only the `linux` block launches anything; the `windows` block is a no-op.
- With `E2E_PLACEMENT=all-local`, both blocks run concurrently on one machine with
  `LocalCheckpointBoard`.
- The `check` job (L14) only runs `expect_published_canvases_match`, over the board.

## 4. Rejected: BRP across the network

One harness driving every machine's BRP would be simpler to write, but:

- BRP has no authentication and can rewrite any `World`, so it must not listen beyond `127.0.0.1`.
- Over 5G (CGNAT) the harness could not reach the Windows peer without a tunnel
  (SSH, Tailscale). A tunnel is a second path between machines, which blurs the X4 question
  "what does Freenet alone give us across NAT?".

A LAN-only SSH tunnel stays possible for debugging, never in the plan.

## 5. Deleted when this lands

- `examples/starboard`: `script.rs`, `script_runner.rs`, `scripted_starboard_step.rs`,
  `status.rs`, `result/`, `scaled_timings.rs`, `time_scale.rs` (if the game's lease timings move
  to an arg), the e2e half of `args.rs`.
- `examples/lobby_room/src/script_runner.rs`, `examples/shared/intent.rs`, `e2e_report/` → tests.
- `crates/freenet-libp2p-bevy-core/src/testing/`: `script_runner.rs`, `timed_script_step.rs`,
  `step_with_deadline.rs`, `script_checkpoint.rs`, `script_verdict.rs`, `e2e_step*.rs`
  (to confirm one by one).
- `scripts/e2e_cross_scenario.sh`.

## 6. Order

1. Core `E2EDriverPlugin` + `SyntheticInputQueue` (glide, buttons, keys) + `E2EMethods`, with
   an in-process `App` test that a glide reaches the target in the expected time.
2. `examples/starboard/src/lib.rs` + `StarboardApp`, `examples/lobby_room/src/lib.rs` +
   `LobbyRoomApp`; both `main.rs` reduced; `track_cursor` /
   `track_pointer` read `PointerLocation` (B4b), checked by hand with a real mouse.
3. `tests/e2e/driver/` + `starboard_e2e` binary; first scenario: one peer creates a room
   and paints, no network.
4. `LobbyE2EMethods`, `E2EId` on lobby UI; `concurrent_strokes_converge` (two peers, no network).
5. Same scenario on `local-gateways`, then `mainnet`.
6. Delete the script machinery (§5).
7. Cross-machine `t.on(..)` on Linux with `all-local`; Windows later.

## 7. Settled (2026-11-14)

- Step A landed (`34553bb`). `RemotePlugin::with_method` is private in 0.20.1: use
  `with_method_main` (lib.rs:591). Code comments are banned (LELE031), so the rule
  **"the driver never calls `Window::set_cursor_position` / `set_physical_cursor_position`"**
  lives here, not in the source. The old script path (`examples/shared/src/intent.rs:147`,
  `friend_intent.rs:210`) does call it, which is why it warps the real cursor; it goes away with §5.
- Keys: each typed character is Pressed then Released on the next frame (C1, `45aa31e`).
- UI targets (C1): no `E2EId` on production UI. Each crate locates its own widgets through its
  existing marker components (`lobby/locate` → `LobbyTarget::{RoomNameInput, CreateRoomButton,
  Room(name)}` → `LocatedNode { centre, disabled }`); the harness then calls `e2e/click_at`.
- Layout (C2, `8afdab2`): the test-owned apps live in their own package `tests/e2e_apps`
  (lib `starboard` = `StarboardE2EMethods`; bin `starboard_e2e`), not under `tests/e2e/starboard/`.
  The harness driver is `tests/e2e/tests/e2e/brp/`; the first scenario `brp_one_peer_paints`
  passes on Xvfb in ~8 s.
- Bug found by the driver (fixed in `8afdab2`): `bevy_input_focus` only focuses a clicked entity
  that has a `TabIndex`, so the room-name, add-friend and custom name inputs never took focus on
  click. They now carry `TabIndex(0)`.
- Hidden display without root: `apt-get download xvfb` + `dpkg -x` into a scratch dir, then
  `Xvfb :99 -screen 0 1280x800x24 -nolisten tcp`; run with `DISPLAY=:99` and the lavapipe ICD.

- Right button erases in `track_pointer`.
- `bevy_remote` 0.20.1 names checked on docs.rs: `RemotePlugin::default().with_method(name, handler)`
  (methods can also be added at runtime through the `RemoteMethods` resource), handler
  `fn(In(params): In<Option<Value>>, world: &mut World) -> BrpResult`,
  `RemoteHttpPlugin::with_address(impl Into<IpAddr>)` / `with_port(u16)`; defaults `127.0.0.1:15702`.
  Built-ins include `world.query`, `world.get_resources`, `world.trigger_event`, `rpc.discover`.
  Each peer gets its own port, chosen by the harness.
- `lobby_room` gets the same lib + wrapper split now (`lobby_room_e2e` binary).
- Windows: deferred to another agent (firewall exe path, gdigrab, runner).
- `bevy_picking` presence: confirm at step 2.

## 8. Status and remaining work (hand-off, 2026-11-15)

### Done (all on `main`)
Every old e2e scenario has a BRP port in `tests/e2e/tests/e2e/brp_*.rs` (23 scenarios), driven by
`tests/e2e/tests/e2e/brp/` against the test-owned apps in `tests/e2e_apps` (`starboard_e2e`,
`lobby_room_e2e`). Networks: fake (default), mainnet, direct-dial. Local-gateways was never
implemented (`NetworkArgs::validate` rejects it) and stays out of scope.

| Batch | Commits | Content |
|---|---|---|
| A–C, E | 34553bb … 9a7b0f9 | core driver, starboard/lobby_room libs, lobby status/locate, e2e apps, BrpNetwork fake/mainnet |
| F1 | 71a29c8 | starboard_e2e, late_join, simultaneous_paint, peer_killed |
| F2 | e7bc834, 6e7b825, 5b144e6 | named keys, join_error, cell_painted; bad_token, history_holder_killed, Backspace check |
| F3 | b28c58b, 979ee79 | Esc-menu/friends locate; leave_rejoin, history_room_gone |
| F4 | b2e5ba8, e4e4c5e, 1ebad07, 5596d80 | e2e/dial, direct_dial/status; direct_dial, known_peers; fix: persist a newly connected peer |
| F5 | 6095aea | friends_reconnect through the real add-friend UI |
| F6 | 7e2f15a, 9874be9, 4df17f8, d99e852, 21deadd, f98c3e9 | lobby_room_join, room_messaging, discovery, ring, history_restore, room_auto_deletion, boot_race_heals, freenet_relaunch |

Bugs the driver found: inputs had no `TabIndex` (click never focused them, 8afdab2); a peer
reached with full addresses was never saved to the peer book (1ebad07).

Lessons that matter for later work:
- Strokes sent while the screen is still `JoiningRoom` are dropped: wait for `InRoom`
  (`StarboardPeer::wait_until_in_room`) before painting.
- History holders must leave the room but keep running (CROSS-OS-PLAN X5).
- Wait 90 s between mainnet runs (tests/README.md:51-58), or the Freenet ring join fails.
- Before an exit that a relaunch depends on, wait until the peer book on disk holds the peer's
  identify addresses (known_peers race, 5596d80).

### Remaining: F7, delete the script machinery (user decision 2026-11-15: delete all, retire e2e-cross)
Do it in three layers, consumers first. Each layer: `just check`, `just test`, all `brp_*` on fake,
one commit, push. Before every deletion, `git grep` that nothing outside the deleted set uses it.

**F7a: tests, justfile, workflow.**
- Delete the old `#[e2e_test]` files in `tests/e2e/tests/e2e/` (the 12 names without `brp_`),
  the old `harness.rs` and helpers only they use; the old targets
  `tests/e2e/tests/{lobby_room_discovery,lobby_room_join,room_messaging,starboard_e2e,starboard_history,starboard_scenarios,freenet_relaunch}.rs`
  and their `[[test]]` entries. Keep `freenet_harness.rs`, `contract_wire.rs`.
- `tests/e2e/tests/common/`: keep only what `brp/` uses (`check_or_deploy`, `lobby_setup`,
  `build_game`, `freenet_node_dir_arg`, …); delete the script-only files.
- `tests/e2e_macros/` goes. `tests/e2e_steps/` goes once `examples/starboard` stops depending on it (F7b).
- `scripts/e2e_cross_scenario.sh`, `scripts/e2e_direct_dial_split.sh`; `scripts/ring_stats.py` and
  the `ring-stats` recipe (old log format) if nothing else uses them.
- justfile: remove e2e-discovery, e2e-ring*, e2e-relaunch*, e2e-join, e2e-room, e2e-starboard,
  e2e-history, e2e-scenarios (+ unused `_e2e-notify-*`); drop `script=info` from the log filter;
  repoint `e2e-ui-no-network` / `e2e-mainnet` to `test(/^brp_/)` (mainnet one at a time, 90 s
  pause); add `e2e-loop TEST RUNS PAUSE`.
- `.github/workflows/e2e-cross.yml`: keep the dispatch trigger, replace the jobs with one
  ubuntu-latest job that fails with "disabled while it is rebuilt on the BRP harness (B8)".
- Rewrite `tests/README.md` for the BRP suite.

**F7b: examples.**
- `examples/starboard`: delete `script.rs`, `script_runner.rs`, `scripted_starboard_step.rs`,
  `status.rs`, `result/`, `scaled_timings.rs`, `time_scale.rs`, `friend_intent.rs`, the e2e half of
  `args.rs`, the `e2e_steps` dependency, and the fake-network lines in `run_starboard`. `main.rs`
  becomes: parse args, start the node, `DefaultPlugins` + `StarboardApp`.
- `examples/lobby_room`: delete `script_runner.rs` and the e2e lines in `main.rs`; make `ring.rs`
  public in its lib and delete the copy in `tests/e2e_apps/src/lobby_room/ring.rs` (use the
  example's). `journey.rs` (a `#[cfg(test)]` UI test) stays.
- `examples/shared`: delete `intent.rs` and `e2e_report/` if unused after.
- Then delete `tests/e2e_steps/`.

**F7c: core and lobby-room `testing`, docs.**
- `crates/freenet-libp2p-bevy-core/src/testing/`: delete `script_runner.rs`, `timed_script_step.rs`,
  `step_with_deadline.rs`, `script_checkpoint.rs`, `script_verdict.rs`, `e2e_step*.rs`,
  `e2e_args.rs`, `__bevy_systems__/run_script.rs`, the script traits, the script parts of
  `orchestrator*.rs` and `peer_builder.rs`, and their `__methods__` (run `lele-lint --sync-methods`).
  Keep `FakeNetworkServer`, `FakeNetworkPlugin`, `PeerMessageRouter`, `CheckpointBoard`,
  `NetworkArgs`/`Network`, and the BRP driver.
- `crates/freenet-libp2p-bevy-lobby-room/src/testing/lobby_step.rs` and its methods go.
- Harness tidy-up: the UI helpers duplicated between `brp/lobby_peer.rs` and
  `brp/direct_dial_peer.rs` (locate, click, type, press_key) move to one shared module.
- Update this plan's §2/§5 to the final layout; mark §6 done.

### Later, not part of F7
- **e2e-cross on BRP (B8):** one test per machine with `t.on(machine, …)`, checkpoints and final
  canvas hashes over the GitHub `CheckpointBoard`, BRP on 127.0.0.1 only. The Windows agent
  owns it (firewall exe path, gdigrab, runner).
- **Product decision (user):** a real user who quits within a few hundred ms of connecting keeps
  the new peer with no addresses in the peer book (same race as known_peers). Option: save a new
  `PreviouslyConnected` entry only once identify delivered its addresses.

### How to run (for the next session)
- Hidden display without root: `apt-get download xvfb` + `dpkg -x` into a scratch dir, run its
  `Xvfb :99 -screen 0 1280x800x24 -nolisten tcp`, then `DISPLAY=:99
  VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`.
- Direct cargo needs `CARGO_TARGET_DIR=$HOME/.cache/cargo-target/freenet-libp2p-bevy-plugin`
  (jemalloc rejects the space in the default path).
- All BRP scenarios on fake, serially:
  `cargo nextest run -p e2e --test e2e -E 'test(/^brp_/)' --run-ignored all --all-features -j 1`
  (the real-network-only ones refuse fake by design). Mainnet: add `E2E_NETWORK=mainnet`, one at a
  time, 90 s apart.
