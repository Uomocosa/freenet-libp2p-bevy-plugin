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
| HD2 | Holder lease | The existing lobby heartbeat **is** the lease: `Heartbeat.presence: Presence { InRoom, HoldingHistory, Quit }`. Renew every 30 s, dead after 90 s (`discovery::LeaseTimings`; the e2e passes values × `E2E_TIME_SCALE`). |
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
- [x] `LeaseTimings` in `discovery::Config`; `HEARTBEAT_TTL` and `REPUBLISH` removed.
- [x] Tests: contract `recreated_room_replaces_old_generation_both_orders`,
      `quit_tombstone_wins_over_older_heartbeat`; `drop_dead` presence table; `merge_room`
      generations; `heartbeat_targets` presences + quit.

### What was done (Step H0)
Contract code changed → `lobby_contract.wasm` rebuilt, so the contract key changes (accepted).
Deviation from the plan: `State::new` keeps its signature (`lease_ttl` defaults to
`LeaseTimings::default().ttl`, `run` overrides it from config) to avoid touching 28 test sites.
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

- [ ] `HistoryLimits { per_process_bytes: 256 MB, per_room_bytes: 64 MB }` in the history
      config (defaults may be lowered; keep them configurable).
- [ ] Writer refuses a snapshot over `per_room_bytes` (error log, nothing stored).
- [ ] `drop_oldest_history` (pure core over held rooms + their byte sizes + leave times →
      rooms to evict): oldest-left first, never the current room, until under
      `per_process_bytes`. Shell: remove the records from the Kad store, remove the room from
      `discovery::state_machine::State.held_rooms`, publish `Presence::Quit` for it.
- [ ] Runs after every history write and on the tick.
- [ ] Tests: unit — eviction order, current room never evicted, stops once under the limit;
      headless — small limit, the held room is evicted, `Quit` is published, and the room
      disappears from another peer's lobby when it was the last holder.
_(agent fills)_

## Order
H0 ✅ → H0b ✅ → H1 → H2 → H2b → H3 → H4.

## Step H3 — History writer/reader in SyncPlugin

- [ ] Writer timer + manifest compare; reader fallback on join (state machine, pure
      reducer + thin system). The fallback replaces `room_sync::state_machine::retry_fetch`'s
      `give_up` (today a joiner silently accepts an empty board): Kad → lobby re-check →
      `discovery::Event::RoomGone` / `HistoryUnavailable` + error screen (HD6).
- [ ] Holders keep their room's records after leaving (HD1); `put_history` sets `expires`
      from the lease instead of `None`.
- [ ] Headless multi-App tests: A,B leave but stay in the game, C joins → same hash;
      all quit → C gets `RoomGone`; holder killed → `RoomGone` after the TTL; last holder
      quits mid-fetch → `RoomGone` within the deadline; recreated name starts empty.

### What was done (Step H3)
_(agent fills)_

## Step H4 — Enable in starboard + e2e

- [ ] Turn on `history` in starboard; local e2e (see `tests/CROSS-OS-PLAN.md`):
      S4a all leave but stay → board restored; S4b all quit → room gone, join error +
      known exit code; S4c last holder killed while a peer joins → error screen.
- [ ] Measure record sizes and put/get latency; write them below.
- [ ] Decide whether `history` becomes default-on.

### What was done (Step H4)
_(agent fills)_

## Open questions
- ~~Who keeps records alive when every room member is offline?~~ Holders (HD1); when the
  last holder quits, the room is gone (HD3).
- ~~Should history be pruned when a new day starts (D6)?~~ No — history belongs to the room
  generation (`created_at`), not the day.
- ~~Abuse/size limits~~ → HD9 / Step H2b. Still measure real record sizes in H4.
- ~~`MAX_ROOMS` fills with dead names~~ → HD8 / Step H0b.
