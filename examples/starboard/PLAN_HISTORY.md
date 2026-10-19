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

## Step H1 — Record keys, chunking codec

- [ ] Key builders, manifest type, chunk/unchunk + hash verify (pure).
- [ ] Tests: round-trip for sizes 0, 1 B, 59 KB, 61 KB, 5 MB; corrupted chunk detected;
      chunk keys differ for different content hashes.
- [ ] **STOP before H3:** the "Open questions" at the bottom of this file must be answered
      by the user first. Ask them; do not guess.

### What was done (Step H1)
_(agent fills)_

## Step H2 — Kademlia put/get commands in the swarm loop

- [ ] `NetCommand::{PutRecord{key,value}, GetRecord{key}}` + `NetEvent::{RecordStored,
      RecordFound{key,value}, RecordNotFound}` (check what `handle_found_record.rs`
      already does and extend it rather than duplicating).
- [ ] Kad server mode for room members; check `MemoryStore` max value size and raise if
      needed (document).
- [ ] Headless test: 3 swarms, A puts, A disconnects, C (connected only to B) gets it.

### What was done (Step H2)
_(agent fills)_

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
- Abuse/size limits: measure in H4; no cap until then.
- `MAX_ROOMS` (256): dead names are never removed from the contract, so a full lobby drops
  new rooms. Later step H5: keep the 256 newest rooms by `created_at` (still a CRDT).
