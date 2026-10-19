# Discovery e2e tests

Real, visual, `#[ignore]`d end-to-end tests for the Freenet room lobby + libp2p mesh.
They spawn real `lobby_room` (or `starboard_gossip`) windows (one embedded Freenet node
each), record the screen, and send the clip to Telegram.

## Run

```
just e2e-discovery   # 3 lobby_room peers: all discover the created room
just e2e-join        # 5 lobby_room peers: discover, join, full mesh
just e2e-room        # 3 lobby_room peers: discover, join, full mesh, all-to-all room gossipsub
just e2e-starboard   # 3 starboard peers paint/erase, plus a 4th that joins late
```

Direct equivalent:

```
cargo nextest run --test lobby_room_discovery --all-features --run-ignored all -- --nocapture
cargo nextest run --test lobby_room_join      --all-features --run-ignored all -- --nocapture
cargo nextest run --test room_messaging       --all-features --run-ignored all -- --nocapture
cargo nextest run --test starboard_e2e        --all-features --run-ignored all -- --nocapture
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

- `scenario.rs`: shared code both tests use. `Scenario::discover_only(3)` / `Scenario::join(5)`.
  `Scenario::starboard(3)` / `Scenario::starboard_late_join(4)` drive the `starboard_gossip`
  binary instead: `bin`, `window_prefix`, `ui` (starboard has none), `scripts` (per-peer
  `--script`) and `late_join` are the knobs.
- UI: every action goes through the on-screen buttons, driven by **real** in-process
  input (`examples/shared/intent.rs::script_input`). `create:<r>` focuses the name
  field and types `<r>` with real `KeyboardInput` events, waiting for the Create button
  to become clickable, then clicks it with a real pointer click (`MouseButtonInput` +
  window cursor). `join:<r>` waits until the room's button appears and clicks it.
  The UI's own click handler sends the `discovery::Command`.
  - the name must be non-empty: an empty name leaves the Create button disabled, and a
    `create` with no name is rejected (an error, never silently ignored).
  - the harness chooses the room name (`--action create:<r>`) and reads it back from
    `lobby created room=<r>`.
  - discovery test: `--ui custom`, which disables `discovery::ui::DefaultUiPlugin` and adds
    the example's replacement `examples/lobby_room/ui.rs::LobbyUi`.
  - join test: `--ui default`, the plugin's default UI.
  - starboard: `--script paint:<n>` / `--script erase:<n>` writes the app's own
    `PendingLocalDraw` messages (the same seam the mouse writes) at fixed cells, so p1
    paints, p2 erases some of it, and every peer must converge on one canvas hash.
- Log markers are the oracle (keep them stable, or update `common/log_parse.rs`):
  - `lobby created room=<r>`
  - `lobby joined room=<r>`
  - `lobby tick rooms=<a,b|none> room=<r|none> members=<k> connected=<c>`
  - `lobby freenet ring ready peers=<n>` — emitted by `attach` **only once the node
    has actually read the lobby** (a local re-Get that never reaches the ring does
    not count); it is the `ring` check.
  - `lobby freenet transport connected peers=<n>` — informational: the transient
    gateway handshake lands within ~100ms and is **not** ring membership.
  - `discovery: lobby contract deployed` — the creator's embedded node acked the
    lobby Put. The deploy ack can arrive after `REQUEST_TIMEOUT`, so the `lobby`
    check also accepts the durable signal (`ring ready … rooms>=1`, i.e. the
    creator read back a lobby already holding its room).
- Checks: `spawn`, `lobby` (creator published), `ring` (every peer read the lobby),
  `discovery` (every peer's rooms lists the room), `join`, `converge`. A node
  that never reaches the ring is reported as `ring`/`isolated=[..]`, not as a
  mystery `discovery` failure. Starboard adds two more:
  - `late_join` (`Scenario::starboard_late_join`): the last peer is spawned only after the
    others are meshed, then every peer must show `connected=N-1` again.
  - `canvas`: after the last `starboard script done` marker, every peer's newest
    `starboard canvas … hash=<h> tiles=<n>` line must agree, with `tiles > 0`
    (budget `CANVAS_BUDGET_SECS` = 30 s).
- Starboard markers (keep them stable, or update `common/log_parse.rs`):
  - `starboard canvas room=<r> hash=<h> tiles=<n> cursor=<x>,<y> color=<c> size=<p>%` (1 Hz)
  - `starboard script done paint|erase strokes=<n>`
- Layering: Freenet only seeds candidates (lobby heartbeat, polled in its own task);
  room membership is libp2p-only — a peer is a member once it is connected AND has sent a
  `/blackboard/pex/1.0.0` handshake for the same room. Handshakes carry the sender's members, so one
  Freenet seed is enough to reach the whole room.
- `lobby mesh room=<r> connected=<c>` is logged the moment the live link count changes; the
  join test's `converge` check = last joiner's press -> every peer at `connected=N-1`,
  budget 500ms (`CONVERGE_BUDGET_MS`).
- Timings: `discovery_ms` = create → every peer lists the room in `rooms`;
  `join_ms` = last joiner spawned → every peer has `room=<r>` and `connected=N-1`.
  Budgets: 120s discovery, 180s join (constants in `common/scenario.rs`).

## Env toggles

- `LOBBY_BUILD_PROFILE` — `dev` (default) or `release`.
- `LOBBY_TRANSPORT` — `both` (default), `tcp`, `quic`.
- `LOBBY_KEEP=1` — leave the windows alive after the run for inspection.
- `LOBBY_RECORD_SECS` — override the ffmpeg capture cap (we stop it early anyway).
- `RUST_LOG` — tracing filter for the example process.
