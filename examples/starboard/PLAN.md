# Starboard — implementation plan

> **Audience:** an AI coding agent with NO prior context. Read this whole file before
> touching anything. Everything you need to know is written here; if something is
> ambiguous, prefer the decision recorded in **§1 Decisions** and write your
> interpretation in the step's "What was done" section.

---

## 0. How to work on this plan (READ FIRST — mandatory rules)

1. **One step per session.** Find the first step whose checkbox list is not fully `[x]`.
   Do only that step. When it is done, stop and report.
2. **Before writing any Rust**, load the `lele-rs` skill (it loads `lele-syntax-rs`;
   read its `references/RATIONALE.md` and `references/SMELLS.md` in full). Also load
   `libp2p` for network steps, `bevy-rs` for Bevy steps, `freenet` for contract steps,
   `definition-state-machine` for any reducer/state-machine code, `definition-tdd` for
   the test loop. Follow the crate's existing conventions: atomic files (one pub item
   per file), `__basic__/` for plain types, `bevy_systems/` for systems, `test_usage`
   tests in every file, `thiserror` errors, no `unwrap/expect/panic/as`, strict clippy
   (see `Cargo.toml` `[lints.clippy]`).
3. **Tooling:** `just`, `lele-lint`, `cargo-nextest` live in `~/.cargo/bin`, which may
   not be on `PATH`. Prefix every command:
   `export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH" && just <recipe> 2>&1`.
   Never pipe `just` output to `tail/head/grep`. Never `git commit --no-verify`.
4. **TDD loop:** write a failing fast test first, make it pass, keep the full fast suite
   (`just test`) green. Slow e2e tests (`#[ignore]`, X11) run only at the step that asks.
5. **Gate before finishing a step:** `just build`, `just clippy`, `just fmt`, `just test`,
   `just lint`, `just bevy-lint` (or `just check`, which runs the prek hooks). If the
   step touched `contract/lobby`, also `just contract`, `just contract-clippy`,
   `just contract-fmt`, `just contract-lint`, `just freenet-harness`.
6. **Commit** with a conventional commit message (`feat(p2p): ...`, `test(starboard): ...`).
   One commit (or a small series) per step.
7. **Fill the step's "What was done" section**: files added/changed, public API added,
   tests added and how to run them, measured numbers, deviations from the plan and why,
   open issues for later steps. Tick every `[ ]` you completed to `[x]`. Commit the
   PLAN.md update together with the step.
8. **HARD RULE (from `tests/README.md`):** peers must discover each other ONLY through the
   Freenet lobby in e2e tests/examples. No mDNS, no hardcoded peers, no `AddKadPeer`
   shortcuts, no passing ids via files/env. In-process headless tests/benchmarks MAY
   wire peers directly (they test transport, not discovery) — say so in a comment.
9. Contract re-keying (changing the lobby contract wasm/params, which changes its key)
   is acceptable — do not add migration logic.
10. If a step turns out to be wrong or impossible, do NOT silently redesign. Write the
    problem in "What was done", propose the change, and stop.
11. **e2e honesty:** if an e2e/X11/Freenet step cannot run in your environment, leave its
    box `[ ]`, write `NOT RUN: <reason>` in "What was done", and stop. A step with an
    unticked e2e box is NOT complete.

---

## 1. Decisions (agreed with the user — do not re-litigate)

| # | Topic | Decision |
|---|---|---|
| D1 | Room transports | Plugin offers **both**: (a) gossipsub `Publish{topic,payload}` / `Subscribe{topic}` / `Unsubscribe`, one topic per room; (b) `Broadcast{payload}` over a **new libp2p stream protocol**: unreliable, fire-and-forget, low-latency, one stream per room member. Selected via `RoomTransport::{Gossipsub, Stream}` in config. |
| D2 | Choosing default | Build both, benchmark (`just compare-starboard`), set plugin default to the winner. Plugin philosophy: many options, **sensible defaults**, "hook it up and it just works". Initial default: `Gossipsub`. |
| D3 | Generic vs app code | Generic room-sync machinery (room broadcast, Lamport clock, snapshot-on-join request/response, tile anti-entropy) lives in the **plugin**. Pixels, shapes, input, rendering live in `examples/starboard/`. |
| D4 | Consistency model | Canvas = **dense, fixed, tiled LWW grid**: 1024×1024 cells, 32×32-cell tiles (32×32 tiles). Cell = `{ lamport: u32, author: u16 (stable 16-bit hash of peer id), color: Option<u8 palette idx> }` (`None` = erased). Merge per cell = keep max `(lamport, author, color)` with `None < Some(_)` — the color tie-break makes a full `(lamport, author)` hash collision harmless (deterministic on every peer). This is a CRDT (commutative, associative, idempotent) under loss/dup/reorder. **No tombstone GC / no `forget()`**: max raw size is bounded (~8 MB). Untouched tiles are never sent. Revisit sparse+GC only if measurement says so. |
| D5 | Lossy repair | Draw ops travel on the fast (possibly lossy) path. **Anti-entropy**: every ~2 s each peer broadcasts per-tile digests (blake3 of the tile's cells, truncated); a peer that sees a differing digest requests that tile (reliable request-response) and merges it. Cursors are lossy, latest-wins, never repaired. |
| D6 | Daily identity | Lobby `Room` gains `created_at: UnixTime` (contract change). `day = (now - created_at) / 86400`. Seed = `blake3(peer_id ‖ created_at ‖ day)`. Shape and color change live on rollover; existing pixels keep their colors. Players cannot change shape/color. **Collision resolution:** sort members by peer id; a later member whose `(shape, color)` is already taken steps to the next free combination (of 72) for that day. **Known limitation (accepted):** rooms are never deleted from the lobby and `created_at` merges by min, so a reused room name keeps its old `created_at` (day counter, seed, history key carry over) and anyone can push `created_at` back to 0. |
| D7 | Shapes / colors | Shapes: `Circle, Star, Square, Triangle, Diamond, Hexagon`. Colors: fixed palette of 12 high-contrast hues (palette index stored in cells). |
| D8 | Canvas geometry | Logical grid 1024×1024, scaled to fit window (letterboxed). Base shape radius = 16 cells. |
| D9 | Size notches | `SIZES = [0.1, 0.2, 0.35, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0]`, start index 5 (1.0). One scroll notch = ±1 index, clamped. Local per player, sent with cursor and ops. |
| D10 | Input | Left click = stamp shape (paint). Hold left = continuous stroke: stamps interpolated between previous and current mouse position, spacing `radius/2`, batched into one `Stroke` op per frame. Right click/hold = same but erase (sets cells to `None`) using the player's shape. Paint overwrites anyone's pixels. |
| D11 | Presence | Each peer streams its cursor (~30 Hz, lossy): position, size index. Others render it as that peer's shape+color outline. Side list shows members with shape/color icon. |
| D12 | Persistence | Phase 1: in-memory only + snapshot from a live member on join. Phase 2 (separate plan `PLAN_HISTORY.md`): Kademlia distributed history so late joiners get the board even if no member answers. |
| D13 | Room size target | 8 peers per room. Benchmarks at 2 and 8 in-process peers, report p50/p99. |
| D14 | Testing bar | Every plugin addition: (1) pure unit tests for reducers/merge, (2) headless in-process two(+)-`App`/two-swarm test, (3) one e2e test in the style of `tests/lobby_room_*.rs`. Starboard final acceptance: e2e where peer A paints, peer B's canvas hash converges to A's. |
| D15 | Example layout | `examples/starboard/` = shared code (as a module tree, included via `#[path]`), plus thin bins `examples/starboard_gossip/main.rs` and `examples/starboard_stream/main.rs` that differ only in `RoomTransport`. `just compare-starboard` benchmarks both, writes `examples/starboard/COMPARE.md`. |

---

## 2. Orientation (what exists today — verify, it may have moved)

- Crate root: `freenet-libp2p-bevy-plugin/` (this repo). Workspace tasks in `justfile`.
- `src/p2p/`: libp2p swarm. `Behaviour<T>` (`src/p2p/behaviour.rs`) already contains
  `request_response`, `exchange`, `kademlia`, `identify`, `ping`, **`gossipsub`** (present
  but no public command uses it yet), `relay`, `relay_client`, `dcutr`, `mdns`.
  Public API: `p2p::Command<T>::{Net(NetCommand), Send{peer_id,payload}}`,
  `p2p::Event<T>::{Net(NetEvent), Message{from,payload}, Error}` in
  `src/p2p/__basic__/enums.rs`; Bevy resources `p2p::Commands<T>` / `p2p::Events<T>`.
  Swarm loop handlers in `src/p2p/swarm_loop/` (`dispatch_command.rs`,
  `handle_gossipsub.rs`, `handle_request_response.rs`, ...).
- `src/discovery/`: Freenet lobby + room state machine (`state_machine/`), `Room`
  struct in `src/discovery/__basic__/structs.rs` (`room_size`, `members`),
  `UnixTime` in `src/discovery/unix_time.rs`, discovery `Command::{Create,Join,Leave}` /
  `Event::{LobbyChanged,Joined,Left,MembersChanged}`.
- `contract/lobby/`: Freenet lobby contract (own workspace; built to wasm with `just contract`).
- `src/plugin/config.rs`: `Config<T> { mode, mdns, marker }`.
- `examples/lobby_room/`: reference example using the default lobby UI.
- `tests/lobby_room_discovery.rs`, `tests/lobby_room_join.rs`, `tests/common/`:
  e2e harness (spawns real windows, records with ffmpeg). Reuse `tests/common/` helpers.

**Step 0 of every session:** `git log --oneline | head -20` and read the "What was done"
sections of completed steps — they override this orientation where they differ.

---

# PART A — Plugin additions

## Step A1 — `created_at` on the lobby Room (contract + plugin)

Goal: rooms carry the unix time they were created; it is immutable once set.

- [ ] Add `created_at: UnixTime` to `Room` in the contract (`contract/lobby/src/...`) and
      in the plugin mirror `src/discovery/__basic__/structs.rs`.
- [ ] Contract merge rule: `created_at` = **min** of both sides (deterministic,
      commutative, idempotent). Set on `Create` from the creator's `UnixTime::now()`.
- [ ] Extend the CRDT-law harness (`tests/freenet_harness.rs`, `just freenet-harness`)
      so the laws cover the new field; extend `tests/contract_wire.rs`.
- [ ] Expose it to apps: `JoinedRoom` (or an accessor) exposes `created_at`.
- [ ] Unit test: merge of two rooms with different `created_at` keeps the min, both orders.
- [ ] Rebuild wasm (`just contract`); the contract key changes — that is fine.
- [ ] Full gate green (incl. contract recipes); `just e2e-join` still passes (run once).

### What was done (Step A1)
_(agent fills: files changed, API, tests + commands, deviations, open issues)_

---

## Step A2 — Room gossipsub: `Subscribe` / `Unsubscribe` / `Publish`

Goal: app can publish a payload to all peers of a room via gossipsub.

- [ ] Add `NetCommand::{SubscribeRoom{room}, UnsubscribeRoom{room}}` and
      `Command::<T>::Publish{ room, payload: T }` (or a dedicated generic room message
      type — decide and document). Topic name = `"<game_id>/room/<room_name>"`.
- [ ] Event: `Event::<T>::RoomMessage { room, from, payload, via: Via::Gossipsub }`
      (introduce `Via` enum now; Stream comes in A3).
- [ ] Wire `handle_gossipsub.rs` + `dispatch_command.rs`. Tune gossipsub for low
      latency in small rooms (heartbeat ~100 ms, `mesh_n_low` suitable for ≤8 peers,
      `flood_publish = true`), all values in a `GossipsubConfig` struct with defaults.
- [ ] Discovery integration: on `discovery::Event::Joined(room)` the plugin auto-
      subscribes; on `Left` auto-unsubscribes (configurable, default on).
- [ ] Check gossipsub `max_transmit_size` (default 64 KiB) and document it; room
      payloads larger than the limit must be rejected with an error, not silently dropped.
- [ ] Test-only seam: a way (`#[cfg(test)]` / dev-only feature) to inject
      `discovery::Event::Joined` + a member set into headless Apps without Freenet, so
      A5/A6/B5 headless tests need no lobby. Starboard bins and `lobby_room` must NOT use it.
- [ ] Unit tests: topic name builder, config defaults.
- [ ] Headless in-process test: 3 swarms on localhost (direct dial allowed — comment
      why), all subscribe, one publishes, the other two receive exactly once.
- [ ] Gate green.

### What was done (Step A2)
_(agent fills)_

---

## Step A3 — Stream protocol `Broadcast` (unreliable, fire-and-forget)

Goal: lowest-latency room fan-out over a dedicated libp2p stream protocol.

- [ ] Use `libp2p-stream` (add the `stream` feature / crate at a version matching
      libp2p 0.56) or a custom `ConnectionHandler`; protocol id
      `/<game_id>/room-stream/1`. Load the `libp2p` skill first.
- [ ] One long-lived outbound stream per connected room member; length-prefixed
      bincode frames; **never block**: bounded per-peer queue, drop oldest when full
      (count drops in a metric).
- [ ] `Command::<T>::Broadcast{ room, payload }` → send to every *current room member*
      (from discovery `Members`, status `Connected`). Incoming → `Event::RoomMessage{..,
      via: Via::Stream}`.
- [ ] Reopen streams on reconnect; close on `Left`/`PeerDisconnected`.
- [ ] Max frame size constant (document it); oversize frames rejected with an error.
- [ ] Unit tests: frame codec round-trip, queue drop policy, oversize frame rejected.
- [ ] Headless test: 3 swarms, broadcast 1000 small msgs, receivers get ≥ 99 % on
      localhost and none duplicated.
- [ ] Gate green.

### What was done (Step A3)
_(agent fills)_

---

## Step A4 — `RoomTransport` config + unified room API

Goal: apps call one API; config chooses the transport.

- [ ] `plugin::Config` gains `room: RoomConfig { transport: RoomTransport, anti_entropy:
      Option<AntiEntropyConfig>, gossipsub: GossipsubConfig, stream: StreamConfig }`
      with `Default` (transport = `Gossipsub`, anti-entropy on). Keep `Config::new`
      working (builder methods for new fields).
- [ ] `Command::<T>::Room{ payload }` (sends to the currently joined room with the
      configured transport). `Publish`/`Broadcast` remain available for explicit use.
- [ ] Two reliability classes for apps: `Room { payload, reliability: Lossy }` uses the
      configured fast transport; `Reliable` uses request-response to each member.
- [ ] Unit tests for config defaults/builders; headless test runs the same scenario
      with each transport.
- [ ] Update `README.md` "Using the plugin" with the room API + defaults.
- [ ] Gate green.

### What was done (Step A4)
_(agent fills)_

---

## Step A5 — Generic room state sync: Lamport clock + snapshot on join

Goal: reusable plumbing so any app can keep a replicated CRDT state per room.

- [ ] Define a trait in the plugin (name per lele conventions), e.g.
      ```rust
      pub trait RoomState: Default + Send + Sync + 'static {
          type Op: p2p::Message;          // fast-path delta
          type Snapshot: p2p::Message;    // full/partial state
          fn apply(&mut self, meta: OpMeta, op: &Self::Op); // must be CRDT-safe
          // OpMeta { lamport: u32, author: u16 } is assigned by the plugin
          fn snapshot(&self) -> Self::Snapshot;
          fn merge(&mut self, snap: Self::Snapshot);    // commutative/idempotent
      }
      ```
      and a `RoomSyncPlugin<S: RoomState>` that owns `Resource<S>` and a `LamportClock`
      resource (`tick()`, `observe(remote)`).
- [ ] On `Joined`: request snapshot (reliable request-response) from one connected
      member (lowest peer id that is `Connected`; retry next on failure/timeout).
      Snapshots are sent **in parts** (one response per part/tile), never one huge
      message; check the request-response codec size limit in `src/p2p/message_codec.rs`
      and document it. Add a size test with a fully painted canvas (all tiles touched). Ops
      received before the snapshot arrives are buffered and applied after merge.
- [ ] App sends ops via a `RoomSync<S>` writer; plugin wraps them with lamport + author
      and sends via the configured room transport.
- [ ] Unit tests: Lamport clock laws; buffering-before-snapshot reducer (pure state
      machine, see `definition-state-machine`).
- [ ] Headless test with a toy `RoomState` (e.g. LWW map `u8 → u32`): A applies ops,
      B joins late, B converges to A.
- [ ] Gate green.

### What was done (Step A5)
_(agent fills)_

---

## Step A6 — Tiled anti-entropy

Goal: heal divergence caused by lossy transports.

- [ ] Extend `RoomState` with optional partition API:
      `type PartId; fn digests(&self) -> Vec<(PartId, u64)>; fn part(&self, id) ->
      Self::Snapshot;`. (Provide a default "single part" impl if simpler.)
- [ ] Every `AntiEntropyConfig.interval` (default 2 s) each peer sends its digest list
      (lossy room transport). Receiver compares; for each mismatching part sends a
      reliable request for that part to the sender; merges reply. Rate-limit
      requests (max K parts in flight per peer, default 8).
- [ ] Unit tests: digest diff reducer.
- [ ] Headless test: toy state, inject 30 % op drop on the transport (test-only fault
      hook), assert all peers converge within 3 intervals after traffic stops.
- [ ] Gate green.

### What was done (Step A6)
_(agent fills)_

---

## Step A7 — Plugin e2e: room messaging over real discovery

- [ ] New `tests/room_messaging.rs` (`#[ignore]`, feature `e2e`, registered in
      `Cargo.toml` like the existing ones) reusing `tests/common/`: 3 `lobby_room`-style
      processes meet via Freenet lobby (HARD RULE), join one room, each sends a room
      message on each transport, all receive. Add `just e2e-room` recipe.
- [ ] Extend `examples/lobby_room` minimally if needed (e.g. a `--room-ping` flag that
      logs received room messages for `tests/common/log_parse.rs`).
- [ ] Run it once; record result.

### What was done (Step A7)
_(agent fills)_

---

# PART B — Starboard example game

Layout (D15):

```
examples/starboard/            # shared module tree, NOT itself an [[example]]
  mod.rs                       # pub mod canvas; identity; input; render; sync; ui; app;
  PLAN.md  PLAN_HISTORY.md  COMPARE.md
  canvas/  identity/  input/  render/  sync/  ui/  app/
examples/starboard_gossip/main.rs   # #[path = "../starboard/mod.rs"] mod starboard; transport = Gossipsub
examples/starboard_stream/main.rs   # same, transport = Stream
```

Register both in `Cargo.toml` as `[[example]]` with `required-features = ["e2e"]`.
Keep game logic pure and Bevy-free where possible (canvas, shapes, rasterizer, identity)
so it is unit-testable; Bevy systems are thin wrappers (see `bevy-rs` skill).

## Step B1 — Daily identity (shape + color)

- [ ] `Shape` enum (D7), `PaletteColor(u8)` newtype + 12-color palette constant.
- [ ] `fn daily_identity(peer_id, created_at, now) -> (Shape, PaletteColor)` per D6
      (pure). `fn day_index(created_at, now) -> u64`.
- [ ] `fn room_identities(members, created_at, now) -> BTreeMap<PeerId, (Shape, PaletteColor)>`
      applying D6 collision resolution (pure).
- [ ] Tests: deterministic for same inputs; changes when day rolls over (`created_at +
      86400`); distribution sanity over 10 000 fake ids (every shape and color used);
      8 members whose raw seeds collide still get 8 distinct identities.

### What was done (Step B1)
_(agent fills)_

---

## Step B2 — Canvas CRDT (dense tiled LWW grid)

- [ ] Types per D4: `Cell`, `Tile` (`[Cell; 32*32]` boxed), `Canvas` (32×32
      `Option<Box<Tile>>`, `None` = never touched), `CellPos`, `TileId`.
- [ ] `Canvas::write(pos, cell)` LWW by `(lamport, author)`; `Canvas::merge_tile`.
- [ ] Implement plugin `RoomState` for `Canvas` (Op = `DrawOp`, Snapshot = list of
      `(TileId, Tile)`, digests per tile with blake3).
- [ ] `author: u16` = stable 16-bit hash of peer id (from `OpMeta`). Per-cell merge key
      is `(lamport, author, color)` (D4) — do NOT rely on the full peer id, cells only
      store the `u16`. Test: two ops with equal `(lamport, author)` but different colors
      merge to the same result in both orders.
- [ ] Tests (proptest recommended): merge commutative / associative / idempotent; apply
      ops in random permutations → identical canvas hash.
- [ ] **Size test**: N = 1k/10k random strokes → report touched tiles, raw bytes,
      bincode+compressed bytes (pick `zstd` or `lz4_flex`; add dep) in "What was done".

### What was done (Step B2)
_(agent fills, include size measurements)_

---

## Step B3 — Rasterizer: shapes, stamps, strokes

- [ ] `DrawOp { kind: Paint(PaletteColor) | Erase, shape, size_idx, from: (f32,f32),
      to: (i16,i16) }` (a click = `from == to`). Coordinates are **integer cells**,
      quantized on the sender before the op is created — no floats on the wire.
- [ ] `fn rasterize(op) -> impl Iterator<Item = CellPos>`: stamps along the segment every
      `radius/2` (radius = `16 * SIZES[size_idx]`, min 1 cell); per-shape coverage
      tests (circle, star 5-point, square, triangle, diamond, hexagon). Pure integer
      math — must be identical on every peer. **No `sin/cos/sqrt` or other float math
      at runtime**: build shapes from precomputed integer vertex tables / masks;
      interpolate stamps with integer (Bresenham-style) stepping.
- [ ] Tests: snapshot-style tests of small rasters per shape; segment with no gaps;
      size 0.1 still covers ≥ 1 cell; clipping at canvas borders.

### What was done (Step B3)
_(agent fills)_

---

## Step B4 — Bevy app: rendering + input (single player, offline)

- [ ] Canvas rendered as one `Image` texture (1024×1024) updated incrementally from
      dirty tiles; letterbox-scaled to window (D8).
- [ ] Input: left press/hold paint, right press/hold erase (D10), scroll notches (D9),
      cursor preview of own shape outline at current size.
- [ ] Mouse → logical coords mapping tested as a pure function.
- [ ] App-based Bevy test (see `bevy-rs`): simulate mouse input → canvas cells changed.
- [ ] Preview test (`_preview` suffix, `lele-bevy-preview` as in existing tests).

### What was done (Step B4)
_(agent fills)_

---

## Step B5 — Networking: lobby → room → synced canvas

- [ ] Reuse the plugin default lobby UI to create/join rooms (copy wiring from
      `examples/lobby_room`). On `Joined`, switch to the board state.
- [ ] Register `RoomSyncPlugin<Canvas>`; local ops go through it; remote ops applied
      and dirty tiles re-uploaded.
- [ ] Presence: cursor message `{ pos, size_idx }` at ~30 Hz lossy via room transport;
      render other peers' cursors as their shape+color outline; side panel with members
      (shape icon + color + display name). Remove cursors of peers that left.
- [ ] Daily rollover recomputed every second from `created_at`.
- [ ] Bins `starboard_gossip` / `starboard_stream` + `just run-starboard <variant>`.
- [ ] Headless in-process test: 2 Apps, A paints scripted strokes, B canvas hash equals A's.

### What was done (Step B5)
_(agent fills)_

---

## Step B6 — `just compare-starboard`

- [ ] Headless benchmark binary/test (no window): N ∈ {2, 8} in-process peers, scripted
      strokes at 60 ops/s per peer for 20 s, for each `RoomTransport`.
- [ ] Measure: op latency p50/p99 (send→apply), bytes/s per peer, % ops dropped before
      anti-entropy, time to canvas-hash convergence after stop.
- [ ] Write a table to `examples/starboard/COMPARE.md` (overwrite each run, include date,
      machine info).
- [ ] Based on results, set the plugin default `RoomTransport` (D2) and justify in
      "What was done".

### What was done (Step B6)
_(agent fills, include table summary + chosen default)_

---

## Step B7 — Starboard e2e acceptance

- [ ] `tests/starboard_e2e.rs` (`#[ignore]`, `e2e`), reusing `tests/common/`: 3 real
      `starboard_<default>` windows meet via Freenet lobby only (HARD RULE), join one
      room; peer A paints scripted strokes (via a test-only `--script` arg), peer B
      erases some; all peers log canvas hash; assert all hashes equal within timeout.
      Record + send the clip like the other e2e tests. `just e2e-starboard`.
- [ ] Late-join case: peer C joins after painting; its hash converges (snapshot path).
- [ ] Run it; record outcome.

### What was done (Step B7)
_(agent fills)_

---

## Step B8 — Polish & docs

- [ ] `examples/starboard/README.md`: how to run, controls, architecture diagram
      (lobby → room → transports → RoomSync → Canvas).
- [ ] Root `README.md`: mention starboard and room API defaults.
- [ ] Revisit open issues collected in previous "What was done" sections; list
      remaining ones here.

### What was done (Step B8)
_(agent fills)_

---

# PART C — Distributed history (later phase)

See **`examples/starboard/PLAN_HISTORY.md`**. Start it only after Part B is complete.
