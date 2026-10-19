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
| X2 | Shipped artifact | Build once on Windows; upload `starboard.exe` + the Freenet node + the checker as one bundle. Later runs reuse it via `bundle_run_id` — no cargo invocation. |
| X3 | Freenet shipping | The node ships with the game; the user never installs/starts Freenet by hand. (Open: bundled `freenet.exe` sidecar — recommended — vs in-process node.) |
| X4 | NAT | No relay of our own. The report separates **Freenet layer** (joined network, saw room/members) from **libp2p layer** (direct connection, how). |
| X5 | History | Kademlia room history (Part C, `examples/starboard/PLAN_HISTORY.md`) lands **before** S4. A player who joined a room and did not quit the game holds and serves its history. |
| X6 | Room deletion | When the last holder quits (clean or not), the room dies and disappears from the lobby. Holders refresh a lease in the lobby contract every 30 s; room dead after 90 s with no fresh lease (both × `E2E_TIME_SCALE`). |
| X7 | Name reuse | A dead room's name is reusable as a fresh room: new `created_at` → new history key, empty board. |
| X8 | Local history file | None. Optional explicit export/import later. |
| X9 | Slow Windows | All waits × `E2E_TIME_SCALE` (default 1, Windows ≈ 4, tune). |
| X10 | Video | Recorded on both machines: Linux ffmpeg x11grab (existing), Windows ffmpeg gdigrab (low fps/resolution). |
| X11 | Runners | Windows self-hosted runner (exists) + this Linux machine registered as `linux-e2e`. |
| X12 | Repo / branch | `freenet-libp2p-bevy-plugin`, branch `feature/cross-machine-e2e`. |

## 2. Test mechanics

- **Step script.** `--script` becomes a `;`-separated list of event-driven steps, e.g.
  `wait-members:2; paint:6; leave; wait-secs:20; rejoin; wait-converged; exit`.
  Every wait is scaled by `E2E_TIME_SCALE`. No cross-machine control channel: the room is the
  synchronisation point.
- **`result.json`** per peer: final canvas hash + tile count, members seen, Freenet layer status
  (gateway, time to join network), libp2p connection kind per peer (tcp/quic/hole-punched/none),
  per-step timings, errors, exit code.
- **Checker.** Reads every machine's `result.json` + logs and prints one table per scenario:
  `expected | actual | PASS/FAIL`.
- **Logs.** Each job uploads `.local-run/<scenario>/` (game logs, Freenet node log,
  `result.json`, video). Read with `gh run download <run-id>`.

## 3. Scenarios (W = Windows peer, L1/L2 = Linux peers)

| # | Scenario | Expected |
|---|---|---|
| S1 | Basic: W paints 6, L1 erases 3 | Same non-empty hash on every peer |
| S2 | Late join: W joins after painting stopped | W converges to the room hash within the scaled budget |
| S3 | W leaves, L1 paints over W's cells, W rejoins | W converges to the new hash; L1 wins those cells (LWW) |
| S4a | All leave the room but stay in the game; a new peer joins | Board restored from Kademlia history, same hash as before |
| S4b | All quit the game; a new peer tries to join | Room gone from the lobby; scripted join gets "room no longer exists" and a known non-zero exit code |
| S4c | Last holder killed while a peer is joining | Joiner shows the error screen and returns to the lobby; recorded in `result.json` |
| S5 | Simultaneous paint on the same cells | Same hash everywhere, deterministic winner |
| S6 | Peer killed (no clean leave) | Others drop it from members within the scaled budget; survivors keep converging |
| S7 | Bad token / room that doesn't exist | Clear error in the log; non-zero exit |

The 5G run also reports, per scenario, both connectivity layers. Layer 1 PASS + layer 2 FAIL is a
valid, expected finding: the follow-up would be Freenet-coordinated hole punching (exchange
observed addresses through the contract, dial simultaneously) — a separate plan.

## 4. Windows minimum-resources profile

- `[profile.ci-min]`: opt-level 0 everywhere, `debug = false`, `incremental = false`.
- `CARGO_BUILD_JOBS=1`, `rust-lld` linker, nextest `-j 1`.
- Windows runs 1 game window + 1 Freenet node; Linux hosts the other peers.

## 5. Workflow — `.github/workflows/e2e-cross.yml` (`workflow_dispatch`)

Inputs: `scenarios` (default all), `bundle_run_id` (optional, skip build), `network_label`
(`lan` | `5g`, reporting only).

Jobs:
1. `build-windows` — only when `bundle_run_id` is empty; uploads the bundle.
2. `peers-windows` and `peers-linux` — run concurrently, share scenario id + room token.
3. `check` — downloads every machine's results, prints the tables, fails on any FAIL.

## 6. Order of work

1. Rename the example to `starboard`.
2. Part C — Kademlia history + holder leases + room deletion, with its local test layers
   (unit → headless multi-App → local e2e S4a/S4b/S4c). All green on Linux first.
3. Ship Freenet with the game (X3).
4. Step script, `result.json`, checker, two-layer connectivity report.
5. S1–S7 green on Linux alone.
6. `ci-min` profile + Windows build-and-bundle job.
7. Register `linux-e2e` runner; two-machine workflow green on `lan`.
8. Windows to 5G; re-run with `bundle_run_id`, `network_label=5g`.
9. Read layer 1 / layer 2 results; decide on Freenet-coordinated hole punching.

## 7. Open

- X3: sidecar `freenet.exe` (recommended) vs in-process node.
- Does the current setup use Freenet's public gateways? The 5G test must run against the public
  network.
- Verify Freenet's current NAT-traversal behaviour (gateways, UDP hole punching, symmetric
  CGNAT) against up-to-date sources before step 8.
