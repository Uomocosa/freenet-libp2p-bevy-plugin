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
| B5 | Curves | Core owns `Curve` (window-space polyline) and `PointerSpeed` (logical px/s, time-based, frame-rate independent). `PointerPath { button, curve, speed }`. Starboard's `Stroke` (board cells) lives in tests and becomes a `Curve` through `starboard/board_to_window`. |
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
2. `examples/starboard/src/lib.rs` + `StarboardApp`; `main.rs` reduced; `track_cursor` /
   `track_pointer` read `PointerLocation` (B4b), checked by hand with a real mouse.
3. `tests/e2e/driver/` + `starboard_e2e` binary; first scenario: one peer creates a room
   and paints, no network.
4. `LobbyE2EMethods`, `E2EId` on lobby UI; `concurrent_strokes_converge` (two peers, no network).
5. Same scenario on `local-gateways`, then `mainnet`.
6. Delete the script machinery (§5).
7. Cross-machine `t.on(..)` on Linux with `all-local`; Windows later.

## 7. Open

- Which mouse button erases in `track_pointer` (assumed Right).
- Exact `bevy_remote` 0.20.1 names (`with_method`, `RemoteHttpPlugin::with_address`, handler
  signature): check docs.rs at step 1.
- Whether `lobby_room` gets the same lib + wrapper split now or later.
- Windows (later): the bundle ships `starboard_e2e.exe`, but X13's firewall allow rule
  matches `C:\e2e\bundle\starboard.exe`. Either install the wrapper under that name and
  path, or add one more allow rule once.
- `bevy_picking` must be in the build. The `ui` feature should pull it in, since the
  `ui_widgets` buttons need it: confirm at step 2.
