# Discovery e2e tests

## Test layers

`just e2e-ui-no-network` runs the `e2e` test target on the fake network: fast,
for reviewing the UI. `just e2e-mainnet` runs the same target on the public
network: the real proof. The network is chosen at run time via `E2E_NETWORK`
(`fake` or `mainnet`); a plain test run with no `E2E_NETWORK` set defaults to
fake, so it can never reach mainnet by accident. Tests pinned with
`network = "direct-dial"` always run on real libp2p with no Freenet node,
whatever `E2E_NETWORK` says.

```
just e2e-ui-no-network starboard_late_join
```

Real, visual, `#[ignore]`d end-to-end tests for the Freenet room lobby + libp2p mesh.
They spawn real `lobby_room` (or `starboard`) windows (one embedded Freenet node
each), record the screen, and send the clip to Telegram.

## Run

```
just e2e-discovery   # 3 lobby_room peers: all discover the created room
just e2e-discovery 5 # the same with 5 peers (E2E_PEERS=2..5)
just e2e-ring 3      # freenet ring health alone: N embedded nodes join the ring
just e2e-ring-loop 3 # the same 5 times, 90 s apart; stop at the first failure, then ring-stats
just e2e-ring-check  # N=1 5 times, then N=3 5 times; stop at the first failure
just e2e-ring-ladder # freenet ring health alone, 2 then 3 then 4 then 5 peers
just e2e-join        # 5 lobby_room peers: discover, join, full mesh
just e2e-room        # 3 lobby_room peers: discover, join, full mesh, all-to-all room gossipsub
just e2e-relaunch   # 1 lobby_room peer, cold then warm with the same node folder and token
just e2e-relaunch-loop # the same up to 5 pairs, 90 s apart; stop at the first failure, then ring-stats both
just e2e-starboard   # 3 starboard peers paint/erase, plus a 4th that joins late
just e2e-history     # the Kademlia history scenarios S4a, S4b, S4c
```

`just ring-stats` reports per-peer timings over `.local-run/freenet_ring-*`
runs: `ring_join_s` (first `connect driver: joined peer`), `first_ring_peers_s`
(first `ring_peers>0`), `lobby_ready_s`, `gate_open_s`, fetch, deployers,
`cached_reconnect` (whether the log shows freenet's peer-cache
`Attempting fast reconnection`). `--scenario freenet_relaunch_cold` and
`--scenario freenet_relaunch_warm` select the relaunch pair's runs.
`--summary --batch "..."` prints one compact paragraph instead (pass count,
`lobby_ready_s`/`ring_join_s` p50 and max, deployers per run, and each failed
run with its reason); `--failed-clips` prints the `clip.mp4` of each failed
run that still has one, tab-separated from its caption.

## Loop pauses and the one batch summary

`e2e-ring-loop`, `e2e-relaunch-loop` and `e2e-ring-check` wait PAUSE seconds
(default 90, `just e2e-ring-loop 3 5 0 30` or `E2E_LOOP_PAUSE_SECS=30`) BETWEEN
runs: never before the first, never after the last. The pause lets the
previous run's peers leave the mainnet ring before the next run joins: with a
90 s pause 8 of 8 runs passed (5 of 24 peers over 30 s to join); back-to-back
runs gave 7 of 8 (11 of 24 peers slow or never joined), because the previous
run's peers had just exited. `e2e-ring-check` takes the same PAUSE and also
waits it between its N=1 and N=3 halves.

Each batch sends exactly ONE Telegram summary at the end: recipe name, N/GAP,
runs passed of total, `lobby_ready_s`/`ring_join_s` p50 and max, deployers per
run, and for a failed run its name and failure reason — plus that run's
`clip.mp4`. Passing runs send nothing else. Per-run Telegram stays silent as
today (empty creds are an inline prefix on the per-run call only, so the
summary still uses the real credentials, read via `telegram-bot-helper`'s
`from_env`). Sending is best-effort: a Telegram failure only prints a warning
and never fails the loop. `e2e-ring-check` sends one combined summary for both
halves; the halves themselves stay silent (`E2E_LOOP_NOTIFY=0`). Direct runs
such as `just e2e-ring 3` are unchanged and still send their own clip and
result.

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

## HARD RULE — no out-of-band peer discovery (mainnet discovery tests only)

The mainnet discovery tests exist to prove that app-level peers know **nothing**
about each other and meet **only** through the Freenet lobby contract, exactly
as if they were on different machines/networks. Do **NOT** add any shortcut, in
the example or the tests:

- Do **not** enable mDNS (the example hardcodes `mdns = false`; keep it that way).
- Do **not** seed Kademlia (`AddKadPeer`) or call `bootstrap()` with known addresses.
- Do **not** use `ReserveRelay`.
- Do **not** hardcode bootstrap peers or peer ids.
- Do **not** pass peer ids/addrs between instances via files, env vars, or the harness.
- Discovery must remain: the creator writes its heartbeat to the Freenet lobby, other
  peers read that row and `Dial`. Nothing else.

Adding any such shortcut invalidates the test even if it makes it pass.

This ban covers the mainnet discovery tests only. The direct-dial tests below
may use known addresses: that is what they are for.

## Direct-dial tests

Three tests prove dialing peers that already know each other. Each pins
`network = "direct-dial"` in its `#[e2e_test]`, which beats `E2E_NETWORK`: the
game starts with `--network direct-dial` (real libp2p, no Freenet node) and a
`--peer-book-file`. `expect_no_freenet_node()` fails the test if the log shows
`lobby freenet node started`. The Orchestrator reads each peer's live addresses
from its log, substitutes peer names in `dial:` and `add-friend:` steps, and
waits to spawn a peer until the address it needs is known. Split runs share
addresses through the checkpoint board instead.

| Test | What it proves |
|---|---|
| `direct_dial` | Alice dials Bob's live address once, with no friend entry. Both see each other connected; Alice exits, Bob sees her leave. |
| `friends_reconnect` | Alice adds Bob as a friend and they connect. Bob restarts with the same identity and book while Alice keeps running, and reconnects with no re-add. Then Alice restarts while Bob runs, and reconnects the same way. The book holds Alice→Bob as friend, Bob→Alice as previously connected. |
| `known_peers` | Alice dials Bob once with no friend entry, so both books learn a previously connected peer. Alice stops, relaunches with the same identity and book while Bob keeps running, and reaches him with no add or dial step. Both sides stay previously connected. |

Run one locally (needs X; `E2E_NETWORK` is ignored):

```
just e2e-ui-no-network direct_dial
```

Or prove the split path on one machine: `scripts/e2e_direct_dial_split.sh`
runs `direct_dial` and `friends_reconnect` with two Orchestrator processes
(`E2E_MACHINE=a/b`, `E2E_PLACEMENT="alice=a,bob=b"`) sharing one local board
directory, then re-runs each test with `E2E_ROLE=check` over the merged
results. It takes one test name or `all`. `known_peers` has no split run; the
plain command above covers it.

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
    (distinct senders, `--room-ping`), `wait-ring-joined` (embedded Freenet node
    has `ring_peers>0`, polled every 500 ms), `wait-lobby-ready` (the lobby was
    fetched), starboard `wait-canvas-nonempty`,
    `wait-canvas-stable` (hash held for 3 s)
  - `sleep:<n>s` — a deliberate pause, never used to sync peers
  - endings: `exit-app`, `crash-app`, `stay`
- UI: `create`/`join` go through the on-screen buttons, driven by **real** in-process
  input (`examples/shared/intent.rs::script_input`): real `KeyboardInput` typing and real
  pointer clicks; the UI's own click handler sends the `discovery::Command`.
  - discovery test: `--ui custom` (`examples/lobby_room/ui.rs::LobbyUi`); join and room
    tests: `--ui default`.
- `on_finish` checks (`common/scenario.rs`):
  - ring: every peer's embedded Freenet node reached the ring (`max_ring_peers>0`
    from `ring_peers=` lines). The script is `wait-ring-joined@120s,
    wait-lobby-ready@60s, exit-app` (cap 5 min): each peer exits soon after its
    own join instead of sleeping a fixed 60 s. Every peer still starts cold
    (temp node folder, new contract key per run).
  - join / room: `converge` = last joiner's press -> every peer at `connected=N-1`,
    budget 500ms (`CONVERGE_BUDGET_MS`), from the `lobby intent pressed` and
    `lobby mesh room=<r> connected=<c>` markers;
  - starboard: every peer's last `starboard canvas … hash=<h> tiles=<n>` agrees, `tiles > 0`;
  - S4a: some holder logged `room history written`, and the late joiner's last canvas hash
    equals the painter's.
  - relaunch: same as ring (each launch passes on its own); the pair only proves
    something together (see below).
- Log markers (keep them stable, or update `common/log_parse.rs`):
  - `script step done step=<i>/<n> <step>`, `script done steps=<n>`,
    `script failed step=<i>/<n> <step> reason=<why>`, `script crash-app`
  - `script checkpoint completed name=<name>` — a peer finished
    `mark-completed:<name>`; `Orchestrator` starts peers waiting on that
    checkpoint once this line appears
  - `lobby tick rooms=<a,b|none> room=<r|none> members=<k> connected=<c>` — libp2p/app layer
  - `freenet node status ring_peers=<n> transport_peers=<m> ring_location=<l>` — embedded Freenet
    node, on change and every 30 s; a gateway-only node reports `ring_peers=0 transport_peers=2`
  - `lobby freenet node waiting ring_peers=<n> …` / `lobby freenet node reachable ring_peers=<n> …` —
    one line per second while `fetch_or_deploy_lobby` waits for the first transport or ring connection
  - `lobby ready … ring_peers=<n>` — the node has read the lobby (ring or gateway)
  - `lobby refresh timed out` / `lobby refresh failed` — the 5 s lobby GET did not answer
  - `starboard canvas room=<r> hash=<h> tiles=<n> cursor=<x>,<y> color=<c> size=<p>%` (1 Hz)
  - `room history written snapshot_bytes=<n> record_bytes=<n> chunks=<n>`
- Layering: Freenet only seeds candidates (lobby heartbeat, polled in its own task);
  room membership is libp2p-only — a peer is a member once it is connected AND has sent a
  `/blackboard/pex/1.0.0` handshake for the same room. Handshakes carry the sender's members, so one
  Freenet seed is enough to reach the whole room.

## Saved-node relaunch (`freenet_relaunch`)

The shipped game starts the node itself and keeps its folder on the player's
disk between launches. This test measures how much a second launch gains: one
peer launches cold (`freenet_relaunch_cold`), exits, waits `E2E_RELAUNCH_GAP_SECS`
(default 10 s) with the app closed, then launches again (`freenet_relaunch_warm`)
with the SAME node folder and the SAME game token, so the second launch has the
same identity and already holds the contract. Both launches run
`wait-ring-joined@120s, wait-lobby-ready@60s, exit-app` and each must pass on its
own; the comparison is in `ring-stats` (cold vs warm) plus the warm log's
`cached_reconnect` (`Attempting fast reconnection`).

Freenet's peer cache expires after 300 s, so GAP below and above 300 mean
different things: below 300 the warm launch can fast-reconnect from cache, above
300 the cache has expired and the warm launch rejoins like a cold one.

## Env toggles

- `LOBBY_BUILD_PROFILE` — `dev` (default) or `release`.
- `LOBBY_TRANSPORT` — `both` (default), `tcp`, `quic`.
- `LOBBY_KEEP=1` — leave the windows alive after the run for inspection.
- `LOBBY_RECORD_SECS` — override the ffmpeg capture cap (we stop it early anyway).
- `RUST_LOG` — tracing filter for the example process.
- `E2E_TIME_SCALE` — scales the lease and history timings of the example (default 1);
  the harness passes its own environment on to the peers.
- `E2E_RELAUNCH_GAP_SECS` — seconds the relaunch test waits with the app closed
  between the cold and warm launches (default 10).
