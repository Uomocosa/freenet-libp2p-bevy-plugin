# Discovery e2e tests

Real, visual, `#[ignore]`d end-to-end tests for the Freenet room lobby + libp2p mesh.
They spawn real `lobby_room` (or `starboard`) windows (one embedded Freenet node
each), record the screen, and send the clip to Telegram.

## Run

```
just e2e-discovery   # 3 lobby_room peers: all discover the created room
just e2e-join        # 5 lobby_room peers: discover, join, full mesh
just e2e-room        # 3 lobby_room peers: discover, join, full mesh, all-to-all room gossipsub
just e2e-starboard   # 3 starboard peers paint/erase, plus a 4th that joins late
just e2e-history     # the Kademlia history scenarios S4a, S4b, S4c
```

Direct equivalent:

```
cargo nextest run --test lobby_room_discovery --all-features --run-ignored all -- --nocapture
cargo nextest run --test lobby_room_join      --all-features --run-ignored all -- --nocapture
cargo nextest run --test room_messaging       --all-features --run-ignored all -- --nocapture
cargo nextest run --test starboard_e2e        --all-features --run-ignored all -- --nocapture
cargo nextest run --test starboard_history    --all-features --run-ignored all -- --nocapture
```

Prerequisites: an X session, `xterm`, `xdotool`, `wmctrl`, `ffmpeg` (with `x11grab`),
`xdpyinfo`, and network access to the public Freenet mainnet. Telegram credentials are
optional (video send is best-effort).

## HARD RULE — no out-of-band peer discovery

These tests exist to prove that app-level peers know **nothing** about each other and
meet **only** through the Freenet lobby contract, exactly as if they were on
different machines/networks. Do **NOT** add any shortcut, in the example or the tests:

- Do **not** enable mDNS (the example hardcodes `mdns = false`; keep it that way).
- Do **not** seed Kademlia (`AddKadPeer`) or call `bootstrap()` with known addresses.
- Do **not** use `ReserveRelay`.
- Do **not** hardcode bootstrap peers or peer ids.
- Do **not** pass peer ids/addrs between instances via files, env vars, or the harness.
- Discovery must remain: the creator writes its heartbeat to the Freenet lobby, other
  peers read that row and `Dial`. Nothing else.

Adding any such shortcut invalidates the test even if it makes it pass.

## How it works (harness = `common/`)

- `scenario.rs`: every e2e test is a `Scenario` = N **scripted peers**
  (`PeerRun { script, start }`), a time `cap`, and an optional `on_finish` check over all
  logs. The harness spawns each peer with `--script "<steps>"` (`{room}` is replaced by the
  run's room), starts late peers when `PeerStart::AfterStepDone { peer, step }` holds, and
  stops when every script ended, one failed, or the cap is reached.
- A peer passes when its script ends as written:
  - `exit-app`: the process exits 0 after logging `script done`;
  - `crash-app`: the process aborts (no quit published, like a crash);
  - `stay`: the process is still running when the run ends (killed by the harness).
- Script grammar (`examples/shared/script/`): `verb[:arg][@Ns]`, comma-separated.
  **Every `wait-*` step must carry `@Ns`, and no other step may.** A wait past its
  deadline fails the script: the peer logs
  `script failed step=<i>/<n> <step> reason=<why>` and exits 3.
  - actions: `create:<r>`, `join:<r>`, `leave:<r>`, starboard `paint-from-to:<x.y>-<x.y>`,
    `erase-from-to:<x.y>-<x.y>` (drawing outside a room fails at once)
  - waits: `wait-lobby-room:<r>`, `wait-join-succeeded:<r>`, `wait-join-failed:<r>`,
    `wait-left:<r>`, `wait-members:<n>` (the peer included), `wait-room-messages:<n>`
    (distinct senders, `--room-ping`), starboard `wait-canvas-nonempty`,
    `wait-canvas-stable` (hash held for 3 s)
  - `sleep:<n>s` — a deliberate pause, never used to sync peers
  - endings: `exit-app`, `crash-app`, `stay`
- UI: `create`/`join` go through the on-screen buttons, driven by **real** in-process
  input (`examples/shared/intent.rs::script_input`): real `KeyboardInput` typing and real
  pointer clicks; the UI's own click handler sends the `discovery::Command`.
  - discovery test: `--ui custom` (`examples/lobby_room/ui.rs::LobbyUi`); join and room
    tests: `--ui default`.
- `on_finish` checks (`common/scenario.rs`):
  - join / room: `converge` = last joiner's press -> every peer at `connected=N-1`,
    budget 500ms (`CONVERGE_BUDGET_MS`), from the `lobby intent pressed` and
    `lobby mesh room=<r> connected=<c>` markers;
  - starboard: every peer's last `starboard canvas … hash=<h> tiles=<n>` agrees, `tiles > 0`;
  - S4a: some holder logged `room history written`, and the late joiner's last canvas hash
    equals the painter's.
- Log markers (keep them stable, or update `common/log_parse.rs`):
  - `script step done step=<i>/<n> <step>`, `script done steps=<n>`,
    `script failed step=<i>/<n> <step> reason=<why>`, `script crash-app`
  - `lobby tick rooms=<a,b|none> room=<r|none> members=<k> connected=<c>`
  - `lobby freenet ring ready peers=<n>` — the node has actually read the lobby
  - `starboard canvas room=<r> hash=<h> tiles=<n> cursor=<x>,<y> color=<c> size=<p>%` (1 Hz)
  - `room history written snapshot_bytes=<n> record_bytes=<n> chunks=<n>`
- Layering: Freenet only seeds candidates (lobby heartbeat, polled in its own task);
  room membership is libp2p-only — a peer is a member once it is connected AND has sent a
  `/blackboard/pex/1.0.0` handshake for the same room. Handshakes carry the sender's members, so one
  Freenet seed is enough to reach the whole room.

## Env toggles

- `LOBBY_BUILD_PROFILE` — `dev` (default) or `release`.
- `LOBBY_TRANSPORT` — `both` (default), `tcp`, `quic`.
- `LOBBY_KEEP=1` — leave the windows alive after the run for inspection.
- `LOBBY_RECORD_SECS` — override the ffmpeg capture cap (we stop it early anyway).
- `RUST_LOG` — tracing filter for the example process.
- `E2E_TIME_SCALE` — scales the lease and history timings of the example (default 1);
  the harness passes its own environment on to the peers.
