# Starboard — distributed history plan (Phase 2)

> Prerequisite: every step of `PLAN.md` Part A and B is `[x]`. Read `PLAN.md` §0
> (working rules) and §1 (decisions) first — they apply here too.

## Goal

A room's board survives when no live member can serve a snapshot (all left, or a late
joiner cannot reach anyone). Every peer that ever joined helps keep the history; new
peers download it on join; it is continuously updated.

## Design (first, simplest version — agreed)

- Store in the existing libp2p **Kademlia** (`Behaviour.kademlia`, `MemoryStore`).
- Record key base: `blake3(game_id ‖ room_name ‖ created_at)` (`created_at` from PLAN A1).
- Value: `RoomState::Snapshot` of touched tiles, `bincode` + compression (same codec as
  PLAN B2), split into chunks ≤ 60 KB: records `key‖"chunk"‖i`, plus a **manifest**
  record `key‖"manifest"` = `{ lamport_max, n_chunks, content_hash }`.
- Writer policy: every 30 s each member reads the manifest; if its own `lamport_max` is
  greater, it `put_record`s chunks then manifest (manifest last). Replication/expiry:
  libp2p defaults, document them.
- Reader policy on join: (1) snapshot from a live member (existing PLAN A5 path);
  (2) if none answers within timeout, `get_record` manifest → chunks → verify hash →
  `RoomState::merge`. Merging is a CRDT, so stale/partial history is harmless.
- Generic: implemented in the plugin as an optional `RoomHistory` feature of
  `RoomSyncPlugin<S>` (config `history: Option<HistoryConfig>`, default off until H4).

## Step H1 — Record keys, chunking codec

- [ ] Key builders, manifest type, chunk/unchunk + hash verify (pure).
- [ ] Tests: round-trip for sizes 0, 1 B, 59 KB, 61 KB, 5 MB; corrupted chunk detected.

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

## Step H3 — History writer/reader in RoomSyncPlugin

- [ ] Writer timer + manifest compare; reader fallback on join (state machine, pure
      reducer + thin system).
- [ ] Headless test: A paints, A and B leave, C joins alone → C gets the board from Kad
      (requires some peer still serving the DHT — use a non-member "keeper" swarm in the
      test and document that real deployments rely on lingering peers).

### What was done (Step H3)
_(agent fills)_

## Step H4 — Enable in starboard + e2e

- [ ] Turn on `history` in starboard; e2e: A paints, quits; B joins later → board appears.
- [ ] Measure record sizes and put/get latency; write them below.
- [ ] Decide whether `history` becomes default-on.

### What was done (Step H4)
_(agent fills)_

## Open questions to settle before/during this phase
- Who keeps records alive when every room member is offline (Kad records live on the
  K closest peers, which may be non-members)? Record TTL vs. re-publish interval.
- Should history be pruned when a new day starts (D6), or kept forever?
- Abuse/size limits: cap total history bytes per room.
