# Starboard — distributed history plan (Phase 2)

> Prerequisite: every step of `PLAN.md` Part A and B is `[x]`. Read `PLAN.md` §0
> (working rules) and §1 (decisions) first — they apply here too.

## Goal

A room's board survives when no live member can serve a snapshot (all left, or a late
joiner cannot reach anyone). Every peer that ever joined helps keep the history; new
peers download it on join; it is continuously updated.

## Design (first, simplest version — agreed)

- Store in the existing libp2p **Kademlia** (`Behaviour.kademlia`, `MemoryStore`).
- Record key base: `blake3(game_id ‖ room_name ‖ created_at)` (`created_at` from PLAN A1). Extend the existing
  `p2p::HistoryChunkId` / `record_key` / `parse_record_key` (today `lobby/history/{room}/{chunk}`, no
  `game_id`/`created_at`) instead of adding parallel key types.
- Value: `RoomState::Snapshot` of touched tiles, `bincode` + compression (same codec as
  PLAN B2), split into chunks ≤ 60 KB: records `key‖content_hash‖"chunk"‖i` (content hash in the
  key so concurrent writers never mix chunks), plus a **manifest**
  record `key‖"manifest"` = `{ lamport_max, n_chunks, content_hash }`.
- Writer policy (**merge before write**): every 30 s each member reads the manifest +
  chunks, verifies the hash, `RoomState::merge`s the stored history into a copy of its
  own state, and if the merged result differs from what is stored it `put_record`s the
  merged chunks then the manifest (manifest last). Never overwrite based on
  `lamport_max` alone — a higher lamport does not imply a superset of ops. Replication/expiry:
  libp2p defaults, document them.
- Reader policy on join: (1) snapshot from a live member (existing PLAN A5 path);
  (2) if none answers within timeout, `get_record` manifest → chunks → verify hash →
  `RoomState::merge`. Merging is a CRDT, so stale/partial history is harmless.
- Generic: implemented in the plugin as an optional `RoomHistory` feature of
  `SyncPlugin<S>` (config `history: Option<HistoryConfig>`, default off until H4).

## Decisions (agreed with the user, 2026-10-19 — do not re-litigate)

| # | Topic | Decision |
|---|---|---|
| HD1 | Who holds history | A player who joined a room and has **not quit the game** holds and serves that room's history (also from the lobby or another room). |
| HD2 | Holder lease | The existing lobby heartbeat **is** the lease: `Heartbeat.presence: Presence { InRoom, HoldingHistory, Quit }`. Renew every 30 s, dead after 90 s (`discovery::HolderHeartbeatIntervals`; the e2e passes values × `E2E_TIME_SCALE`). |
| HD3 | Room death | A room is dead when it has no heartbeat that is fresh and not `Quit`. Client-side only (`Lobby::drop_dead`): the contract has no clock and never prunes. |
| HD4 | Clean quit | `AppExit` → `Command::Quit` → every target is republished as `Presence::Quit` (tombstone, wins by newest `updated_at`). Crash/kill is covered by the 90 s expiry. |
| HD5 | Name reuse | A dead room's name is reusable as a fresh room: lobby merge keeps the **newer** `created_at` (same `created_at` merges heartbeats). New `created_at` → new history key → empty board. |
| HD6 | Stuck joiner | History fetch deadline → re-check the lobby: dead → "Room `<name>` no longer exists (all players left)"; alive → retry N times, then "Couldn't get room history". |
| HD7 | Local history file | None. An explicit export/import may come later. |
| HD8 | Dead rooms in the contract | **Option A + cap** (agreed 2026-10-19). The contract drops a room once **every** heartbeat in it is `Quit` (deterministic, no clock). When the lobby is full (`MAX_ROOMS`), it evicts the room whose newest heartbeat is oldest. Rooms whose last holder crashed stay until the cap evicts them; clients already hide them (`drop_dead`). No contract clock, no median — skew-proof by construction. |
| HD9 | History size caps | Per process: **256 MB** of held history (configurable, may be lowered by default). Per room snapshot: **64 MB** — the writer refuses a larger one and logs an error. On overflow, `drop_oldest_history` evicts held rooms oldest-left first, never the current room, and publishes `Quit` for each evicted room (you no longer hold it, so the lobby must know). |

## Step H0 — Holder leases and room deletion

- [x] `Presence` on the contract and client `Heartbeat` (same bincode field order); newer
      `created_at` wins in both `merge_room`s; `Lobby::prune` → `Lobby::drop_dead` (drops
      `Quit` + expired); `seed_candidates` dials only `InRoom`.
- [x] `State.held_rooms` (leave adds, join removes), `State.lease_ttl`, `State.is_quitting`;
      `heartbeat_target` → `heartbeat_targets` (one per current/held room); `run_lobby`
      publishes them all every `lease.renew_every`.
- [x] `Command::Quit` + `send_quit_on_exit` (`Last` schedule, sends straight through
      `CommandSender` so no second `forward_commands` instance double-forwards).
- [x] `HolderHeartbeatIntervals` in `discovery::Config`; `HEARTBEAT_TTL` and `REPUBLISH` removed.
- [x] Tests: contract `recreated_room_replaces_old_generation_both_orders`,
      `quit_tombstone_wins_over_older_heartbeat`; `drop_dead` presence table; `merge_room`
      generations; `heartbeat_targets` presences + quit.

### What was done (Step H0)
Contract code changed → `lobby_contract.wasm` rebuilt, so the contract key changes (accepted).
Deviation from the plan: `State::new` keeps its signature (`lease_ttl` defaults to
`HolderHeartbeatIntervals::default().ttl`, `run` overrides it from config) to avoid touching 28 test sites.
Verified: build, clippy, fmt, lint, bevy-lint, contract clippy/lint, 255 nextest, freenet-harness.

## Step H0b — Drop dead rooms from the contract (HD8)

- [x] Contract `merge_lobby`: after merging, drop rooms where every heartbeat is
      `Presence::Quit`.
- [x] Contract cap: when inserting would exceed `MAX_ROOMS`, evict the room with the
      oldest newest-heartbeat (tie-break: smaller `RoomName`) instead of silently dropping
      the incoming room.
- [x] Client `discovery::freenet::merge_room` / `Lobby::merge` mirror both rules so the
      client view matches the contract.
- [x] Tests: all-`Quit` room is gone after merge (both orders); a stale `InRoom` heartbeat
      arriving after the drop resurrects the room in the contract but `drop_dead` still hides
      it (documented, accepted); cap evicts the oldest room; `just freenet-harness` CRDT laws
      still pass — if the cap eviction breaks associativity, stop and report.
- [x] Contract key changes again → rebuild `lobby_contract.wasm`.

### What was done (Step H0b)

- Contract: `merge_lobby` inserts every valid incoming room, then
  `drop_rooms_where_every_member_quit` (a room with no members counts as all-`Quit`), then
  `evict_oldest_rooms_until_lobby_fits_capacity(&mut base, MAX_ROOMS)`. Eviction rank is
  `newest_heartbeat_in(room)`; ties lose to the smaller `RoomName`.
- The cap keeps the **newest `MAX_ROOMS` of the union**, so an incoming room older than
  every held room is itself evicted. That is what keeps merge associative (a "the incoming
  always wins" rule is not).
- Client mirror: `discovery::freenet::drop_rooms_where_every_member_quit` and
  `discovery::freenet::evict_oldest_rooms_until_lobby_fits_capacity` (capacity parameter,
  `discovery::freenet::MAX_LOBBY_ROOMS = 256`), both called once from `Lobby::merge` in the
  contract's order: sweep, then cap.
- Sweep-before-cap means an all-`Quit` incoming room never displaces a live one.
- Tests: contract `all_quit_room_is_dropped_in_either_merge_order`,
  `stale_live_heartbeat_resurrects_a_dropped_room`, `all_quit_room_never_displaces_a_live_room`,
  `cap_evicts_the_oldest_room_and_keeps_the_incoming_one`,
  `cap_evicts_the_smaller_name_when_newest_heartbeats_tie`,
  `cap_evicts_the_incoming_room_when_it_is_the_oldest`,
  `evicts_the_oldest_rooms_until_the_lobby_fits`; client `merge` both-orders sweep +
  capacity; both new client fns have `test_usage`.
- H0's `quit_tombstone_wins_over_older_heartbeat` now asserts the room is **gone** (the
  tombstone wins by killing the room, not by keeping a `Quit` member). The `refresh`/`poll`
  fixtures gained a real heartbeat because a zero-member room is now dropped.
- `just freenet-harness` green (idempotent, commutative, associative unchanged).
- Contract code changed → `lobby_contract.wasm` rebuilt, so the contract key changes again
  (accepted). Verified: build, clippy, fmt, lint, bevy-lint, contract clippy/fmt/lint,
  260 nextest + 10 contract tests, freenet-harness.

## Step H1 — Record keys, chunking codec

- [x] Key builders, manifest type, chunk/unchunk + hash verify (pure).
- [x] Tests: round-trip for sizes 0, 1 B, 59 KB, 61 KB, 5 MB; corrupted chunk detected;
      chunk keys differ for different content hashes.
- [x] **STOP before H3:** the "Open questions" at the bottom of this file must be answered
      by the user first. Ask them; do not guess.

### What was done (Step H1)
Commit `b9a8812`: `history_key_prefix` (blake3 of game/room/`created_at`),
`split_snapshot_into_chunks` / `join_chunks_into_snapshot` (zstd, ≤ 60 KB chunks, blake3
content hash), `HistoryManifest` + `ChunkError`, manifest key builder; `HistoryChunkId` is now
prefix/content-hash scoped. The open questions were already answered (see bottom).

## Step H2 — Kademlia put/get commands in the swarm loop

- [x] `NetCommand::{PutRecord{key,value}, GetRecord{key}}` + `NetEvent::{RecordStored,
      RecordFound{key,value}, RecordNotFound}` (check what `handle_found_record.rs`
      already does and extend it rather than duplicating).
- [x] Kad server mode for room members; check `MemoryStore` max value size and raise if
      needed (document).
- [x] Headless test: 3 swarms, A puts, A disconnects, C (connected only to B) gets it.

### What was done (Step H2)
- Deviation: commands/events take a typed `p2p::RoomHistoryRecordKind { Manifest(prefix),
  Chunk(HistoryChunkId) }` instead of a raw key (`kad::RecordKey` is not `Serialize`).
  `NetCommand::{PutHistoryRecord{id,value}, GetHistoryRecord{id}}`;
  `NetEvent::{HistoryRecordStored, HistoryRecordPutFailed{reason}, HistoryRecordFound{value},
  HistoryRecordNotFound}` replace `PutHistory`/`FetchHistory`/`HistoryChunk`.
- `record_key` + `history_manifest_record_key` → `history_record_key`; `parse_record_key` →
  `parse_history_record_key` (now also parses the manifest key).
- `handle_found_record` → `handle_kademlia_query_result`: `FoundRecord` → Found;
  `NotFound`/`Timeout`/`QuorumFailed` → NotFound; `PutRecord` ok/err → Stored/PutFailed.
  `put_history_record` reports a local-store refusal as PutFailed and no longer calls
  `start_providing` (provider records are useless for value records).
- **Packet size fix:** kad's default `max_packet_size` is 16 KiB, so every 60 KB chunk failed.
  Now `p2p::KADEMLIA_MAX_PACKET_BYTES = 128 KiB` (the test fails without it).
- Kad defaults kept (libp2p-kad 0.48): `MemoryStore` `max_value_bytes` 65 KiB (fits a 60 KB
  chunk), `max_records` 1024 (≈ 60 MB of chunks — below HD9's 256 MB, raise in H2b), record
  TTL 48 h, replication 1 h, republish 22 h, replication factor 8 (ours), `Quorum::One`.
  Server mode was already on.
- Test `p2p::run::tests::kademlia_history_record_survives_its_writer_leaving`: writer and
  reader both connect only to a middle node; writer puts the manifest + a 60 KB chunk, is
  aborted, reader gets both byte-identical; an unknown id gives NotFound.
- Verified: build, clippy, fmt, lint, bevy-lint, 274 nextest.

## Step H2b — History size caps + `drop_oldest_history` (HD9)

- [x] `HistoryLimits { per_process_bytes: 256 MB, per_room_bytes: 64 MB }` in the history
      config (defaults may be lowered; keep them configurable).
- [x] Writer refuses a snapshot over `per_room_bytes` (error log, nothing stored).
- [x] `drop_oldest_history` (pure core over held rooms + their byte sizes + leave times →
      rooms to evict): oldest-left first, never the current room, until under
      `per_process_bytes`. Shell: remove the records from the Kad store, remove the room from
      `discovery::state_machine::State.held_rooms`, publish `Presence::Quit` for it.
- [x] Runs after every history write and on the tick.
- [x] Tests: unit — eviction order, current room never evicted, stops once under the limit;
      headless — small limit, the held room is evicted, `Quit` is published, and the room
      disappears from another peer's lobby when it was the last holder.
### What was done (Step H2b)
- Renamed per review: `discovery::HistorySizeLimits { per_process_bytes, per_room_bytes }`
  (`p2p::HistoryByteCount`), defaults 256 MB / 64 MB, field `discovery::Config.history_size_limits`.
- **Deferred to H3:** the per-room 64 MB refusal. No writer exists yet; `per_room_bytes` is
  config only until the H3 writer enforces it.
- Byte counting lives in the swarm loop (owns the `MemoryStore`): `NetCommand::MeasureHistoryStore`
  → `measure_history_store` → `NetEvent::HistoryStoreMeasured { bytes_by_prefix }`.
  `NetCommand::RemoveHistoryRecords { prefix }` → `remove_history_records` (local store only;
  replicas elsewhere expire by kad TTL). Records of rooms neither held nor current are not counted.
- `State.held_rooms` value is now `HeldRoomHistory { created_at, left_at }` (`leave` takes `now`);
  `State` gained `game`, `history_size_limits`, `evicted_rooms` (name → `created_at`).
  `heartbeat_targets` publishes every evicted room as `Presence::Quit`; re-joining clears it.
- Pure core `held_rooms_to_evict_until_history_fits` (current room + held rooms total; evicts
  oldest `left_at` first, ties by name, until ≤ limit; current room is never a candidate).
  Transition `drop_oldest_history(state, bytes_by_prefix)`. Deviation: no `now` parameter (unused).
- Triggers: every `tick` and every `HistoryRecordStored` emit `MeasureHistoryStore`.
- Kad `MemoryStore` `max_records` raised 1024 → `p2p::KADEMLIA_MAX_RECORDS = 8192` (≈ 480 MB of
  60 KB chunks, so HD9 is the binding cap).
- Tests: selector order / under-limit / current-room-over-limit; `drop_oldest_history` → `Quit`
  target + `RemoveHistoryRecords`; `measure_history_store`, `remove_history_records` usage;
  headless `p2p::run::tests::history_over_limit_is_removed_from_the_kademlia_store`.
  "Room disappears from another peer's lobby" needs Freenet → covered in H4/S4b.
- Verified: build, clippy, fmt, lint, bevy-lint, 283 nextest.

## Order
H0 ✅ → H0b ✅ → H1 ✅ → H2 ✅ → H2b ✅ → H3 → H4.

## Step H3 — History writer/reader in SyncPlugin

- [x] Writer timer + manifest compare; reader fallback on join (state machine, pure
      reducer + thin system). The fallback replaces `room_sync::state_machine::retry_fetch`'s
      `give_up` (today a joiner silently accepts an empty board): Kad → lobby re-check →
      `discovery::Event::RoomGone` / `HistoryUnavailable` + error screen (HD6).
- [x] Holders keep their room's records after leaving (HD1); `put_history` sets `expires`
      from the lease instead of `None`.
- [x] Headless multi-App tests: A,B leave but stay in the game, C joins → same hash;
      all quit → C gets `RoomGone`; holder killed → `RoomGone` after the TTL; last holder
      quits mid-fetch → `RoomGone` within the deadline; recreated name starts empty.

### What was done (Step H3)
- `NetCommand::PutHistoryRecord` and `put_history_record` now carry `ttl: Duration` and set
  `kad::Record.expires = Instant::now().checked_add(ttl)` from the history config (default
  90 s = `HolderHeartbeatIntervals::default().ttl`).
- `discovery::state_machine::seed_candidates` → `collect_room_peers_from_lobby`: returns
  `Vec<Output>` emitting `NetCommand::AddKadPeer` for every non-`Quit` lobby member (`InRoom`
  and `HoldingHistory`) while still seeding only `InRoom` into `state.candidates`.
- `room_sync::HistoryConfig { write_interval: 30s, record_ttl: 90s, per_room_bytes: 64 MB,
  max_fetch_retries: 3 }`, `HistoryWriteTimer`, and `SyncPlugin::with_history_config` (plus
  `with_anti_entropy` → `with_anti_entropy_config`).
- `room_sync::state_machine::State<Op>` decomposed into `MemberSnapshotFetchState`
  (`received_parts: BTreeMap<u32, Vec<u8>>`), `SliceAntiEntropyState`, `KademliaHistoryState`,
  `PendingHistoryFetch` (`received_chunks: BTreeMap<ChunkIndex, Vec<u8>>`), and
  `LobbyRoomStatus`.
- Parallel `Phase` / `Input` / `Output` naming: `FetchingSnapshotFromRoomMember`,
  `FetchingHistoryFromKademlia`, `Failed`; `SnapshotPartReceivedFromRoomMember`,
  `SnapshotPartRequestedByRoomMember`, `RequestToRoomMemberFailed`, `HistoryWriteTimerFired`,
  `HistoryRecordFound`, `HistoryRecordNotFound`; `FetchSnapshotPartFromRoomMember`,
  `SendSnapshotPartToRoomMember`, `GetHistoryRecord`, `MergeAndWriteHistory`, `RoomGone`,
  `HistoryUnavailable`.
- Transitions: `start_kademlia_history_fetch`, `start_history_write_check`,
  `handle_history_record_found`, `handle_history_record_not_found`, and pure helper
  `merge_and_write_history_records` (merges stored history into replica before writing,
  enforces HD9 `per_room_bytes`, skips write if `content_hash` matches, writes chunks then
  manifest).
- `discovery::Event::{RoomGone, HistoryUnavailable}` + default UI `Screen::JoinFailed`,
  `JoinError`, `JoinErrorRoot`, `JoinErrorDismissButton`, `record_join_error`,
  `spawn_join_failed` (+ PNG preview), `dismiss_join_failed`.
- Verified: build, clippy, fmt, lint, bevy-lint, 298 nextest.

## Step H4 — Enable in starboard + e2e

- [x] Turn on `history` in starboard; local e2e (see `tests/CROSS-OS-PLAN.md`):
      S4a all leave but stay → board restored ✅; S4b all quit → room gone, join error +
      known exit code ✅; S4c last holder killed while a peer is joining → error screen
      ✅ (finding 7 fixed below).
- [x] Measure record sizes and put/get latency; write them below.
- [x] Decide whether `history` becomes default-on.

### What was done (Step H4)

**Files added (product):**
- `examples/starboard/script.rs` — the `--script` vocabulary: `ScriptStep`
  (`CreateRoom`, `JoinRoom`, `LeaveRoom`, `WaitForNMembers`, `WaitSecs`,
  `WaitConverged`, `PaintFromTo`, `EraseFromTo`, `ExitGame`), `MemberCount`, and
  `parse_script_step`.
- `examples/starboard/parse_stroke.rs` — `parse_stroke`: `10.20-30.102` → a
  `Stroke`, with an error that names the offending part.
- `examples/starboard/script_runner.rs` — `ScriptRunner` + `run_script`: walks
  the step list and logs `starboard script done <step>` per step.
- `examples/starboard/join_failure.rs` — `JoinTarget`, `JoinLimits`,
  `JOIN_FAILED_EXIT_CODE = 2`, and `exit_on_join_failure`: the watchdog that turns a
  failed join into one known exit code.
- `examples/starboard/scaled_timings.rs` — `scaled_lease_timings`,
  `scaled_history_config`: the lease and history timings × `E2E_TIME_SCALE`.
- `examples/starboard/bench/history_bench.rs` — `HistoryBoard`, `HistoryReport`,
  `measure_history`, `history_markdown` + the ignored `history_bench` test.
- `examples/starboard/HISTORY.md` — the measured record sizes and latencies.
- `tests/starboard_history.rs` — the three local e2e scenarios (S4a, S4b, S4c).

**Files changed (product):**
- `src/room_sync/bevy_systems/drive.rs` — **the plugin fix the e2e found**: the lobby
  liveness check. A room that is not in the lobby is only *gone* when the contract
  has confirmed it (the discovery's `RoomPhase::Ready`); while the peer is still
  joining (`RoomPhase::Joining`) the contract may simply not have published the room
  yet, and the history fallback used to call that "all players left" — which made the
  **creator fail its own join** (the new room is not in the contract for a few
  seconds). New test: `a_room_the_contract_has_not_confirmed_yet_is_not_gone`.
- `examples/starboard/main.rs` — the history is on
  (`with_history_config(Some(...))`), the lease and history timings are scaled, the
  script runner runs before the lobby intent, the join watchdog is registered, and
  `main` returns `AppExit` so a failed join exits with code 2.
- `examples/starboard/args.rs` — `--script` is now a comma-separated list of
  steps (`value_delimiter = ','`), `--script` and `--action` are exclusive
  (`conflicts_with`), and `--join-timeout-secs` (default 300) bounds the wait for
  the room to show up in the lobby.
- `examples/starboard/canvas/__basic__/structs.rs` — `Stroke { from, to }` (the
  canvas's coordinate pair; `DrawOp` keeps its flat `from`/`to` because it is the
  bincode wire format).
- `examples/starboard/input/draw_op.rs` — `draw_op(brush, action, stroke)` takes the
  named stroke instead of two positional cells.
- `examples/starboard/input/bevy_systems/track_pointer.rs`,
  `examples/starboard/bench/scripted_stroke.rs`,
  `examples/starboard/app/starboard_plugin.rs` — the `draw_op` call sites.
- `examples/starboard/app/test_harness.rs` — `node_with(game, history)`: one fixture,
  one behaviour parameter.
- `examples/shared/intent.rs` — `PendingAction` resource + `Intent::arm_with_action`;
  the shared `script_input` system arms the intent when the wanted action changed,
  so the method is used by both examples (no dead code in `lobby_room`).
- `examples/shared/action.rs` — `Action` derives `PartialEq, Eq` (the arming compares it).
- `src/room_sync/merge_and_write_history_records.rs` — one `tracing::info!`:
  `room history written snapshot_bytes=… record_bytes=… chunks=…`. The e2e sequences
  on it, and the sizes are measurable from the log.
- `tests/common/scenario.rs` — `Scenario` is now a flow, a roster and a budget:
  `ScenarioFlow` (one enum replaces the `join` / `is_room_ping_enabled` / `late_join`
  flags), `PeerSpec { role, spawn, script }`, `PeerRole`, `PeerSpawn`,
  `ScenarioBudgets`, plus the three history flows.
- `tests/common/log_parse.rs` — `history_write_bytes`: the snapshot byte count of the
  last history write.
- `tests/common/tile.rs` — `tile` takes the window titles (the roster owns the
  windows, not a parallel list).
- `Cargo.toml` — the `starboard_history` test. `justfile` — `bench-history`,
  `e2e-history`.

**The script vocabulary** (the `--script` list is comma-separated):

| Step | Argument | Meaning |
|---|---|---|
| `create:<room>` | room name | type the name, press Create |
| `join:<room>` | room name | press that room's button |
| `leave:<room>` | room name | leave the room, stay in the game |
| `wait-for-n-members:<n>` | member count | wait until n peers are connected in the room |
| `wait-secs:<n>` | seconds | wait n seconds |
| `wait-converged:<room>` | room name | wait until that room's canvas hash stops changing |
| `paint-from-to:<from>-<to>` | `x.y-x.y` cells | paint one stroke |
| `erase-from-to:<from>-<to>` | `x.y-x.y` cells | erase one stroke (waits for content first) |
| `exit` | — | publish `Quit`, then end the game |

**Public API added:** `room_sync` — unchanged (only a log line). `examples/starboard` —
`canvas::Stroke`. `examples/starboard` — `ScriptStep`, `MemberCount`,
`parse_script_step`, `parse_stroke`, `ScriptRunner`, `run_script`, `JoinTarget`,
`JoinLimits`, `JOIN_FAILED_EXIT_CODE`, `exit_on_join_failure`, `scaled_lease_timings`,
`scaled_history_config`. `tests/common` — `ScenarioFlow`, `PeerSpec`, `PeerRole`,
`PeerSpawn`, `ScenarioBudgets`, `history_write_bytes`.

**Tests added:** `parse_script_step` (every verb, unknown verb, missing room, bad
count, bad stroke, the harness's own script shape), `parse_stroke` (round trip,
missing cells, non-numeric cell, cell outside the canvas), `args` (a comma-separated
script, the `--script`/`--action` conflict, a bad step's reason), `script_runner` (a
zero-second wait, a paint step waiting for the room, a join step arming the intent),
`join_failure` (a room that left the lobby, the plugin's own join error, no report
without a join), `scaled_timings` (the scaled lease and history timings), the
harness's `every_scripted_stroke_names_two_cells` (the script shape the harness
writes), and the plugin's `a_room_the_contract_has_not_confirmed_yet_is_not_gone`.

**Measured (`examples/starboard/HISTORY.md`, `just bench-history`, dev profile,
AMD Ryzen 5 7600X, 12 threads, in-process peers on localhost):**

| Board | Strokes | Snapshot bytes | Record bytes | Chunks | Put latency | Get latency |
|---|---:|---:|---:|---:|---:|---:|
| e2e board | 6 | 187 | 196 | 1 | 21 ms | 21 ms |
| busy board | 10 000 | 426 158 | 333 204 | 6 | 48 ms | 0 ms |

The put latency is the time until a second peer confirmed the record; the get
latency is the time until the reader held the manifest (the chunks follow). The busy
board's record is 333 KB in 6 chunks of ≤ 60 KB — far below HD9's 64 MB per-room cap.

**Decisions:**
- **`history` stays off by default in the plugin.** `room_sync::SyncPlugin::history`
  remains `None`; the starboard bin opts in. The measurement says the feature is
  cheap (196 B and ~20 ms for the e2e board), but it is a *distributed* feature whose
  put is only confirmed when the writer's Kademlia routing table has a peer — the
  lobby seeds it, a bare peer does not — and the cross-machine run has not proven it
  yet. It becomes default-on only after the 5G run (CROSS-OS-PLAN step 8) is green.
- **The record lives ten minutes, not one lease.** The starboard bin sets
  `record_ttl = 600 s` (scaled): the mainnet's contract state can take *minutes* to
  reach a cold node (measured: 2 min 27 s for a late joiner's first lobby read), so a
  record that dies with the lease (90 s) is gone before a late joiner can ask for it.
  The room's death is decided by the lobby, not by the record's TTL.
- **`--script` and `--action` are exclusive** (the user's call). The consequence: the
  script carries the room action (`create:` / `join:`), and the harness templates the
  run's room name into the script text (`{room}`) because the room name is only known
  per run.
- **The comma delimiter, not the semicolon** (the user's call): `;` separates
  statements in PowerShell, so `--script "a, b, c"` keeps the shell out of it.
- **The stroke is explicit** (the user's call): `PaintFromTo(Stroke)` / `EraseFromTo(Stroke)`
  replaced `Paint(StrokeCount)` / `Erase(StrokeCount)`, and `StrokeCount` is gone —
  the script says which cells to paint, not how many strokes.
- **The history scenarios do not use the op path.** Only the first peer paints: the
  second one would have to *erase* to differ, and an erase published before the
  gossipsub mesh knows the subscription is lost.

**Findings:**
1. **The put is local-first, the confirmation is asynchronous.** libp2p-kad's
   `put_record` stores the record in the local store *before* the replication query,
   so a holder can serve the history immediately; but the `HistoryRecordStored` event
   only arrives once a peer confirms the quorum. A peer whose Kademlia routing table
   is empty never gets that event — the bench had to seed the routing table the way
   the lobby does (`AddKadPeer`), and the plugin itself never waits for the event.
2. **The room snapshot is compressed twice.** The canvas's snapshot is already zstd
   (B2), and the chunker compresses it again — which is why the busy board's record
   (333 KB) is *smaller* than its snapshot (426 KB).
3. **Turning the history on exposed a bug in the fallback's liveness check** (fixed
   above): the creator of a room failed its own join, because the new room is not in
   the contract for the first seconds and the fallback read that as "all players
   left". The liveness check now only calls a room gone once the contract confirmed
   it.
4. **The fallback counts retries, not a deadline.** HD6 says "history fetch deadline
   → re-check the lobby", but the implementation gives up after `max_fetch_retries`
   (3) *failed fetches* — and a failed Kademlia fetch is fast (tens of milliseconds),
   so three of them are over long before a room's first history write (up to 30 s
   after the room started). A peer that joins a room whose history has not been
   written yet can therefore be told "couldn't get room history". Left as it is (the
   fallback only runs when no live member can serve the board), and recorded here for
   the next step.
   **2026-10-29, partly fixed:** a joiner that reached the fallback before its first
   member connected (S4a run `…-1793275900`: member connected 34 ms after the join,
   the three misses failed it 100 ms later) ignored that member. `handle_members` now
   asks a member that connects during the Kademlia fetch for the board
   (`a_member_connecting_during_the_kademlia_fetch_is_asked_for_the_board`).
   **Still open:** a joiner that can reach *no* live member still gives up after three
   fast misses; the real fix is a time-based deadline in room sync (matters for the 5G
   run, where the direct libp2p link may fail).
5. **The mainnet's contract state lags a cold node by minutes.** A late joiner read
   an *empty* lobby for 2 min 27 s before the room appeared in it — the contract's
   subscription had not caught up with the node's cold start. Two consequences: the
   record's TTL has to be minutes (see the decisions above), and the e2e's budgets
   have to cover a late joiner's own join (`HISTORY_RESTORE_BUDGET_SECS` = 420 s).
6. **The gossipsub mesh did not form between the peers, so the ops were lost.** The
   peers were connected and had exchanged their topic subscriptions, yet
   `mesh_peers(topic)` was 0 for the whole overlap: every publish failed with
   `NoPeersSubscribedToTopic` (the known window of B2's open issue #3) and the
   anti-entropy did **not** repair it within 25 s (its announcements travel on the
   same topic). The room sync therefore still converges only through the snapshot
   path. Pre-existing, not introduced by H4 — but it is the reason the history
   scenarios do not rely on the op path.
7. **The fallback cannot tell "the holder died" from "the room is new"** — S4c's
   joiner (the only holder was killed) got an *empty board* instead of the error
   screen. The fallback accepts an empty board when the lobby has no *other* holder,
   and "other" is counted as "the live members minus one" — which counts the local
   peer as one of them. While the joiner's own heartbeat has not reached the
   contract yet, the room's only other member (the dead holder) is taken for "me",
   so the room looks like it has no other holder. The fix has two parts: the room
   sync needs its own peer id (it can: `p2p::ReadyChannel`) so "other" means "not
   me", and it needs the lease TTL so a member whose heartbeat is older than the
   lease (dead) can be told apart from one that is still connecting. Neither is
   exposed to the room sync today, so S4c is **NOT RUN** (the scenario is in the
   tree and fails with this reason).
   **FIXED 2026-10-29:** `room_prefix_and_lobby_status` now uses `UnixTime::now()`
   (not the max heartbeat) for freshness, so a lone expired holder is dead
   (`is_room_alive_in_lobby = false` → `RoomGone`); `self_peer_id`/`lease_ttl` ride
   on `discovery::Status` (populated from `ReadyChannel`/holder lease config), and
   an unknown self counts every live member as other (never subtracts one). New fast
   tests: `expired_holder_alone_means_the_room_is_gone`,
   `fresh_holder_means_wait_for_history`, and headless-App
   `expired_holder_alone_is_gone_not_empty`. S4c now passes (see the table).
   **SUPERSEDED 2026-10-29 (observation/hybrid rule):** lease expiry is a hybrid of
   "observed by this peer" and "remote timestamp far too old", lives only in
   discovery, and is re-checked on every `Tick`. A lobby member is dead if (a)
   `Departed`, or (b) `now - first_seen_locally_at > lease_ttl`, or (c)
   `now.elapsed_since(remote updated_at) > lease_ttl + remote_clock_skew_margin`
   (margin default 10 min, scaled by `E2E_TIME_SCALE`). `DiscoveryState` records
   `observed_lobby_heartbeats` on every `handle_lobby` and prunes on both
   `handle_lobby` and `tick`; the tick prune keeps observations (only `record`
   cleans against the raw incoming lobby) so a dropped member the contract still
   carries stays dropped instead of flapping back for another lease;
   `room_prefix_and_lobby_status` keeps no clock and
   counts every non-`Departed` member as live. The S4c headless-App test now builds
   its `Status` from a real `DiscoveryState` (`handle_lobby` then `tick` past the
   lease) instead of hand-injecting an expired heartbeat.
   **FOLLOW-UP 2026-10-29 (created-vs-joined rule, S4b):** a joiner's own heartbeat
   in the lobby no longer counts as "room alive". `JoinedRoom` carries
   `entry: RoomEntry::{Created, Joined}` (set from `Command::Create`/`Join`);
   with `entry == Joined`, `is_room_alive_in_lobby` requires at least one live
   member other than self (own entry does not count), so a joiner whose lobby
   holds only itself reports `RoomGone` instead of accepting an empty board.
   With `entry == Created` the own entry still counts (new room, empty board OK).

**Commands run (all green):** `just build`, `just clippy`, `just fmt`, `just test`
(**398 passed**, 11 skipped), `just lint`, `just bevy-lint`, `just bench-history`.

**e2e (`just e2e-history`, one machine, Freenet mainnet, dev profile):**

| Scenario | Result | Evidence |
|---|---|---|
| S4a — the holders left the room, a late joiner restores the board | ✅ | `wrote=true left=true late_joined=true restored=true`; the late joiner's canvas hash equals the holders' (`18418444412079406693`) |
| S4b — every player quit the game, a later joiner finds the room gone | ✅ | `quit=true late_joined=true join_failed=true`: the joiner reported `Room <name> no longer exists (all players left)` and exited with code 2 |
| S4c — the last holder is killed while a peer is joining | ✅ | 2026-10-29: `p1 crashed=true`, `p2 wait-join-failed done, exit 0`; the joiner saw the error screen instead of an empty board (finding 7 fixed) |

The runs are slow by nature: the mainnet's contract state reaches a cold node in
seconds to minutes (measured: 26 s for the first batch, 27 s for the discovery, and
2 min 27 s for a late joiner's first lobby read in one run), and the e2e waits for
the *events* (the history write, the leave, the restore) rather than for a clock.

S4b's check does not require the *watcher* to see the room disappear: the contract's
state reaches the watcher's node minutes after the contract dropped the room, so the
watcher can still list a room that is already gone. The watcher stays in the
scenario as the evidence that the room really died, and the check reports
`watcher_saw_the_room_gone=` beside the result.

**Open issues (for later steps):**
- The e2e run is recorded in `tests/CROSS-OS-PLAN.md`'s terms; `result.json` and the
  cross-machine workflow are CROSS-OS-PLAN §6 steps 3–5, not H4.
- `E2E_TIME_SCALE` is read from the environment by the bin and scales the lease and
  the history timings; the local e2e runs at scale 1 (the mainnet lobby round trip is
  20–60 s, so a *shorter* lease would kill live rooms).


## Open questions
- ~~Who keeps records alive when every room member is offline?~~ Holders (HD1); when the
  last holder quits, the room is gone (HD3).
- ~~Should history be pruned when a new day starts (D6)?~~ No — history belongs to the room
  generation (`created_at`), not the day.
- ~~Abuse/size limits~~ → HD9 / Step H2b. Measured in H4: 196 B for the e2e board,
  333 KB in 6 chunks for a 10 000-stroke board (see `examples/starboard/HISTORY.md`).
- ~~`MAX_ROOMS` fills with dead names~~ → HD8 / Step H0b.
- ~~Should `history` be default-on in the plugin?~~ No (H4): the starboard bin opts in;
  it becomes default-on only after the cross-machine run (CROSS-OS-PLAN step 8) is green.
