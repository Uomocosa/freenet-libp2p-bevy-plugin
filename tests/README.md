# Discovery e2e tests

Real, visual, `#[ignore]`d end-to-end tests for the Freennet room board + libp2p mesh.
They spawn real `lobby_room` windows (one embedded Freenet node each), record the
screen, and send the clip to Telegram.

## Run

```
devenv tasks run lobby:e2e-discovery   # 3 peers: all discover the created room
devenv tasks run lobby:e2e-join        # 5 peers: discover, join, full mesh
```

Direct equivalent:

```
cargo nextest run --test lobby_room_discovery --all-features --run-ignored all -- --nocapture
cargo nextest run --test lobby_room_join      --all-features --run-ignored all -- --nocapture
```

Prerequisites: an X session, `xterm`, `xdotool`, `wmctrl`, `ffmpeg` (with `x11grab`),
`xdpyinfo`, and network access to the public Freenet mainnet. Telegram credentials are
optional (video send is best-effort).

## HARD RULE — no out-of-band peer discovery

These tests exist to prove that app-level peers know **nothing** about each other and
meet **only** through the Freenet `directory` board contract, exactly as if they were on
different machines/networks. Do **NOT** add any shortcut, in the example or the tests:

- Do **not** enable mDNS (the example hardcodes `mdns = false`; keep it that way).
- Do **not** seed Kademlia (`AddKadPeer`) or call `bootstrap()` with known addresses.
- Do **not** use `ProvideLobby` / `FindLobby` / `ReserveRelay`.
- Do **not** hardcode bootstrap peers or peer ids.
- Do **not** pass peer ids/addrs between instances via files, env vars, or the harness.
- Discovery must remain: the creator writes its presence to the Freenet board, other
  peers read that row and `Dial`. Nothing else.

Adding any such shortcut invalidates the test even if it makes it pass.

## How it works (harness = `common/`)

- `scenario.rs`: shared code both tests use. `Scenario::discover_only(3)` / `Scenario::join(5)`.
- UI: every action goes through the on-screen buttons. `--action create` / `--action join:<r>`
  is a scripted press (`examples/lobby_room/intent.rs::press_once`) that marks the button
  `Interaction::Pressed`; the UI's own click handler sends the `discovery::Command`.
  A join press waits until the room's button appears in the list. The UI picks the room
  name, and the harness reads it back from `lobby created room=<r>`.
  - discovery test: `--ui custom`, which disables `discovery::ui::DefaultUiPlugin` and adds
    the example's replacement `examples/lobby_room/ui.rs::LobbyUi`.
  - join test: `--ui default`, the plugin's default UI.
- Log markers are the oracle (keep them stable, or update `common/log_parse.rs`):
  - `lobby created room=<r>`
  - `lobby joined room=<r>`
  - `lobby tick catalogue=<a,b|none> room=<r|none> members=<k> connected=<c>`
- Layering: Freenet only seeds candidates (board presence, polled in its own task);
  room membership is libp2p-only — a peer is a member once it is connected AND has sent a
  `/blackboard/pex/1.0.0` hello for the same room. Hellos carry the sender's members, so one
  Freenet seed is enough to reach the whole room.
- `lobby mesh room=<r> connected=<c>` is logged the moment the live link count changes; the
  join test's `converge` check = last joiner's press -> every peer at `connected=N-1`,
  budget 500ms (`CONVERGE_BUDGET_MS`).
- Timings: `discovery_ms` = create → every peer lists the room in `catalogue`;
  `join_ms` = last joiner spawned → every peer has `room=<r>` and `connected=N-1`.
  Budgets: 120s discovery, 180s join (constants in `common/scenario.rs`).

## Env toggles

- `LOBBY_BUILD_PROFILE` — `dev` (default) or `release`.
- `LOBBY_TRANSPORT` — `both` (default), `tcp`, `quic`.
- `LOBBY_KEEP=1` — leave the windows alive after the run for inspection.
- `LOBBY_RECORD_SECS` — override the ffmpeg capture cap (we stop it early anyway).
- `RUST_LOG` — tracing filter for the example process.
