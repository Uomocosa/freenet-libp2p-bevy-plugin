# Cross-OS e2e plan — starboard on Linux + Windows (LAN, then 5G)

## 0. Goal

Prove the plugin and the starboard example work between two real machines:

1. Linux + Windows on the **same network** (`lan`).
2. Then, **without rebuilding anything**, Windows moved to **mobile 5G** (`5g`) and every
   scenario re-run.

The 5G run answers one question: **what does Freenet alone give us across NAT?** We do **not**
host our own relay — that would defeat the purpose of Freenet.

## 1. Decisions (agreed — do not re-litigate)

| # | Topic | Decision |
|---|---|---|
| X1 | Binary name | `starboard_gossip` example → `starboard` (`starboard.exe` on Windows). |
| X2 | Shipped artifact | Build once per OS: `bundle-windows` (`starboard.exe` + `e2e_check.exe`, Freenet is in-process) and `bundle-linux` (`starboard`, built on `linux-e2e`). Later runs reuse both via `bundle_run_id` — no cargo invocation. |
| X3 | Freenet shipping | Decided: in-process node (`freenet` crate, `examples/shared` `start_embedded_freenet_node`); `--freenet-node-dir` keeps its config/data/logs. |
| X4 | NAT | No relay of our own. The report separates **Freenet layer** (joined network, saw room/members) from **libp2p layer** (direct connection, how). |
| X5 | History | Kademlia room history (Part C, `examples/starboard/PLAN_HISTORY.md`) lands **before** S4. A player who joined a room and did not quit the game holds and serves its history. |
| X6 | Room deletion | When the last holder quits (clean or not), the room dies and disappears from the lobby. Holders refresh a lease in the lobby contract every 30 s; room dead after 90 s with no fresh lease (both × `E2E_TIME_SCALE`). |
| X7 | Name reuse | A dead room's name is reusable as a fresh room: new `created_at` → new history key, empty board. |
| X8 | Local history file | None. Optional explicit export/import later. |
| X9 | Slow Windows | All waits × `E2E_TIME_SCALE` (default 1; raise it only for a specific slow run, since 4 can take an hour to fail). The game scales its lease and history timings; `scripts/e2e_cross_scenario.sh` scales each script's `@Ns` deadlines. |
| X10 | Video | Each machine records its own desktop (Linux ffmpeg x11grab, Windows ffmpeg gdigrab, low fps/resolution) and sends the clip to Telegram. |
| X11 | Runners | Both online on this repo since 2026-11-07: `windows-e2e` (`C:\actions-runner`) and `linux-e2e` (`~/actions-runner-freenet`). Never a service: they must run in the logged-in desktop session, screen unlocked. A runner offline for about 14 days is deleted by GitHub and must be registered again. |
| X12 | Repo / branch | `freenet-libp2p-bevy-plugin`, branch `main`. |
| X13 | Windows firewall | At most one allow prompt, ever. Rules match by program path, so the workflow always runs the game from `C:\e2e\bundle\starboard.exe`, which has an inbound allow rule. Never run it from a per-run path. |
| X14 | Public repo | `workflow_dispatch` only; fork PR approval is `all_external_contributors`. Artifacts are public, so videos are never uploaded as artifacts and go to Telegram through the TELEGRAM_BOT_TOKEN and TELEGRAM_CHAT_ID repo secrets; Freenet node folders are never uploaded, only their logs. |

## 2. Test mechanics

- **Step script.** `--script` is a comma-separated list of `verb[:arg][@Ns]` steps, e.g.
  `wait-members:3@420s, paint-from-to:200.300-260.300, wait-canvas-stable@60s, exit-app`.
  Every wait carries a deadline. The mainnet discovery scenarios (S1–S7) share
  nothing across machines: the room is the synchronisation point. The
  direct-dial scenario instead shares live addresses and checkpoints through
  the GitHub checkpoint board (commit statuses, one status per address);
  mainnet discovery tests may not.
- **`result.json`** per peer: final canvas hash + tile count, members seen, Freenet layer status
  (gateway, time to join network), libp2p connection kind per peer (tcp/quic/hole-punched/none),
  per-step timings, errors, exit code.
- **Checker.** Reads every machine's `result.json` + logs and prints one table per scenario:
  `expected | actual | PASS/FAIL`. The example `cargo run -p e2e_check -- --results-dir <dir> --peer <name>... --expect converge --expect min-members:N --expect may-crash:<user> --expect join-must-fail:<user>` does this.
- **Logs.** Each peers job uploads `results-windows` / `results-linux`, one `<peer>/` folder each
  (game log, Freenet node logs, `result.json`). Videos go to Telegram, not artifacts.
  Read with `gh run download <run-id>`.

## 3. Scenarios (W = Windows peer, L1/L2 = Linux peers)

| # | Scenario | Expected | Test |
|---|---|---|---|
| S1 | Basic: W paints 6, L1 erases 3 | Same non-empty hash on every peer | `starboard_e2e` (`just e2e-starboard`) |
| S2 | Late join: W joins after painting stopped | W converges to the room hash within the scaled budget | `starboard_late_join` (`just e2e-starboard`) |
| S3 | W leaves, L1 paints over W's cells, W rejoins | W converges to the new hash; L1 wins those cells (LWW) | `starboard_leave_rejoin` (`just e2e-scenarios`) |
| S4a | All leave the room but stay in the game; a new peer joins | Board restored from Kademlia history, same hash as before | `starboard_history_restore` (`just e2e-history`) |
| S4b | All quit the game; a new peer tries to join | Room gone from the lobby; scripted join gets "room no longer exists" and a known non-zero exit code | `starboard_history_room_gone` (`just e2e-history`) |
| S4c | Last holder killed while a peer is joining | Joiner shows the error screen and returns to the lobby; recorded in `result.json` | `starboard_history_holder_killed` (`just e2e-history`) |
| S5 | Simultaneous paint on the same cells | Same hash everywhere, deterministic winner | `starboard_simultaneous_paint` (`just e2e-scenarios`) |
| S6 | Peer killed (no clean leave) | Others drop it from members within the scaled budget; survivors keep converging | `starboard_peer_killed` (`just e2e-scenarios`) |
| S7 | Bad token / room that doesn't exist | Clear error in the log; non-zero exit | `starboard_bad_token` (`just e2e-scenarios`) |

The 5G run also reports, per scenario, both connectivity layers. Layer 1 PASS + layer 2 FAIL is a
valid, expected finding: the follow-up would be Freenet-coordinated hole punching (exchange
observed addresses through the contract, dial simultaneously) — a separate plan.

## 4. Windows minimum-resources profile

- `[profile.ci-min]`: opt-level 0 everywhere, `debug = false`, `incremental = false`.
- `CARGO_BUILD_JOBS=1`, `rust-lld` linker, nextest `-j 1`. Never more than one worker on this PC.
- The PC has 7.4 GB RAM. If the build runs out of memory, set `build_host=github`.
- Windows runs 1 game window + 1 Freenet node; Linux hosts the other peers.

## 5. Workflow — `.github/workflows/e2e-cross.yml` (`workflow_dispatch`)

Inputs: `scenarios` (`s1` | `direct-dial`), `bundle_run_id` (optional, skip builds),
`network_label` (`lan` | `5g`, reporting only), `time_scale`, `build_host`
(`self-hosted` | `github`), `record_video`.

Jobs for `s1` (unchanged: the mainnet proof, no board, no placement):

1. `prepare` — generates the room token and each peer's script.
2. `build-windows` and `build-linux` — only when `bundle_run_id` is empty; upload the bundles.
3. `peers-windows` and `peers-linux` — run concurrently, share scenario id + room token.
4. `check` — runs `e2e_check.exe` on `windows-e2e` over every machine's results, prints the
   tables, fails on any FAIL.

Jobs for `direct-dial` (routine cross-OS run, no Freenet node):

1. The same builds, plus the compiled `e2e` test program joins each bundle as
   `e2e-direct-dial(.exe)`, so later runs need no cargo.
2. `peers-windows-direct` (`E2E_MACHINE=w`) and `peers-linux-direct`
   (`E2E_MACHINE=l`) run concurrently with `E2E_PLACEMENT=alice=w,bob=l`,
   `E2E_BOARD=github`, and `E2E_GAME_BIN` pointing at the bundled `starboard`.
   Each runs `direct_dial` then `friends_reconnect` under its own board
   namespace (`E2E_BOARD_RUN_ID=<run>-<attempt>-<test>`).
3. `check-direct` merges both machines' `results-*-direct` artifacts and runs
   the test program with `E2E_ROLE=check` (and `E2E_RESULTS_DIR` at the merged
   results) over both tests.

The board is commit statuses on the workflow commit (`statuses: write`): one
status per address (`<run>/peer/<name>/<index>`, each `addr/p2p/id` fits the
140-character description limit, two do not) and one per checkpoint
(`<run>/checkpoint/<name>`, state `success`). LAN addresses may appear on the
board; never keys. Reads are cached for 10 s, and remote-peer lookup only runs
while a local peer still waits to spawn. S1 stays the mainnet proof.

## 6. Order of work

- [x] 1. Rename the example to `starboard` (2026-10-29: `examples/starboard_gossip/*` →
      `examples/starboard/`, `[[example]] name = "starboard"`, `cargo build --example starboard` green).
- [x] 2. Part C — Kademlia history + holder leases + room deletion, with its local test layers
      (unit → headless multi-App → local e2e S4a/S4b/S4c). All green on Linux first
      (2026-10-29: fast 398 passed; S4a/S4b/S4c PASS on mainnet, finding 7 fixed).
- [x] 3. Ship Freenet with the game (X3) (2026-10-29: in-process node; --freenet-node-dir).
- [x] 4. Step script, `result.json`, checker, two-layer connectivity report (2026-10-29: step script from Part C; result.json via --result-json; e2e-check example; connection kind = /p2p-circuit -> relayed, else quic/tcp; hole-punched = relayed then direct).
- [x] 5. S1–S7 green on Linux alone.
   - 2026-10-29 live: 8/9 green — S1 219s, S2 142s, S3 86s, S4a 199s, S4b 236s, S4c 48s, S6 163s, S7 332s; S5 FAILS: canvases diverge when both peers paint right after a join (anti-entropy did not repair). Headless repro: examples/starboard/app/starboard_plugin.rs two_apps_painting_the_same_cells_right_after_joining_converge. Root cause: concurrent dialing of a peer's addresses created short-lived duplicate connections; gossipsub's one-time subscription exchange rode a closed one, so the joiner never learned the creator's room topics (312x NoPeersSubscribedToTopic live). Fix: DIAL_CONCURRENCY_FACTOR = 1 in dial_peer.
   - 2026-11-06 live: 9/9 green on mainnet — S1 45s, S2 129s, S3 144s, S4a 273s, S4b 140s, S4c 52s, S5 152s, S6 55s, S7 266s. S6 needed a test fix: the crashing peer now waits for all members, so it cannot crash before the last peer joins.
6. `ci-min` profile + Windows build-and-bundle job (written 2026-11-07, not yet run).
7. Two-machine workflow green on `lan`: S1 first, then port S2–S7 to `scripts/e2e_cross_scenario.sh`.
8. Windows to 5G; re-run with `bundle_run_id`, `network_label=5g`.
9. Read layer 1 / layer 2 results; decide on Freenet-coordinated hole punching.

## 7. Open

- ~~X3: sidecar `freenet.exe` (recommended) vs in-process node.~~
- S2, S3, S4 start a peer after another peer's step (`PeerStart::AfterStepDone`), which needs a
  local orchestrator. Across machines this has to become script waits on room state.
- `ci-min` runs every dependency at opt-level 0, while local runs use 1. Check the game still
  keeps up on the Windows PC; if not, this is what to change first.
- Does the current setup use Freenet's public gateways? The 5G test must run against the public network. Yes: `skip_load_from_network = false` loads https://freenet.org/keys/gateways.toml (freenet 0.2.132 config.rs:80).
- Verify Freenet's current NAT-traversal behaviour (gateways, UDP hole punching, symmetric
  CGNAT) against up-to-date sources before step 8.
