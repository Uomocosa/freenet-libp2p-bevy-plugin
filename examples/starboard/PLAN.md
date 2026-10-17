# Starboard — implementation plan

> **Audience:** an AI coding agent with NO prior context. Read this whole file before
> touching anything. Everything you need to know is written here; if something is
> ambiguous, prefer the decision recorded in **§1 Decisions** and write your
> interpretation in the step's "What was done" section.

---

## 0. How to work on this plan (READ FIRST — mandatory rules)

1. **One step per session.** Find the first step whose checkbox list is not fully `[x]`.
   Do only that step. When it is done, stop and report.
2. **Before writing any Rust**, load the skills in **§0.1 Required skills** — in full,
   including the two files it names (`lele-syntax-rs/references/RATIONALE.md` and
   `references/SMELLS.md`); the `skill` tool does NOT inline them. Follow the crate's
   existing conventions: atomic files (one pub item per file), `__basic__/` for plain
   types, `bevy_systems/` for systems, `test_usage` tests in every file, `thiserror`
   errors, no `unwrap/expect/panic/as`, strict clippy (see `Cargo.toml` `[lints.clippy]`).
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

### 0.1 Required skills (ALWAYS load — no exceptions)

Load every skill whose row matches the step **before** naming, designing, reviewing, or
editing anything. `lele-rs` is mandatory for *any* Rust; the others are mandatory for the
step types listed. The `skill` tool advertises skills by description but does **not**
inline their reference files — `read` the ones named.

| Skill | Load when | Notes |
|---|---|---|
| `lele-rs` | **always** (any Rust) | entrypoint; load first |
| `lele-syntax-rs` | **always** (any Rust) | then `read` `references/RATIONALE.md` **and** `references/SMELLS.md` in full — the gate, not the skill file |
| `bevy-rs` | any Bevy work: systems, UI, plugin wiring (A8, B4, B5, all UI) | Bevy 0.19 uses `Message`/`MessageWriter`; plugin struct + delegate split |
| `definition-state-machine` | reducers / `state_machine/` transitions (A1, A5, A6, A8) | one `update`; transitions **return** their outputs |
| `definition-tdd` | every step's test loop | fast red → full fast suite → slow gate |
| `freenet` | contract work (A1, PLAN_HISTORY) | `freenet-contract-harness`, CRDT merge rules, key == `blake3(wasm)` |
| `libp2p` | network room messaging (A2, B5) | `SwarmBuilder`, gossipsub |
| `serde-rs` | wire types / snapshots (A1, A2, A5) | bincode is **positional** — field order must match the contract |
| `criterion-rs` | benchmarks (B6, `just bench-starboard`) | groups, `black_box` |
| `opencode-git-workflow` | commits (§0.6) | conventional commits; **only ever on an explicit user command** |

Rule: if a step touches a row's topic and that skill is not loaded, **STOP and load it
first**. Loading a skill is cheap; skipping one is a correctness bug.

---

## 1. Decisions (agreed with the user — do not re-litigate)

| # | Topic | Decision |
|---|---|---|
| D1 | Room transport | **Gossipsub only**, one topic per room: `Publish{topic,payload}` / `Subscribe{topic}` / `Unsubscribe`. The stream-protocol `Broadcast` variant is dropped: `libp2p-stream` publishes only `-alpha` releases (no stable) and a hand-rolled `ConnectionHandler` is not worth it for a ≤8-peer room. |
| D2 | Transport choice | Moot — there is no second transport. `just bench-starboard` still measures the gossipsub path (op latency, bytes/s, drop %, convergence; 2 vs 8 peers) but no longer picks a default; the plugin keeps the A2 gossipsub tuning for small rooms. |
| D3 | Generic vs app code | Generic room-sync machinery (room publish, Lamport clock, snapshot-on-join request/response, tile anti-entropy) lives in the **plugin**. Pixels, shapes, input, rendering live in `examples/starboard/`. |
| D4 | Consistency model | Canvas = **dense, fixed, tiled LWW grid**: 1024×1024 cells, 32×32-cell tiles (32×32 tiles). Cell = `{ lamport: u32, author: u16 (stable 16-bit hash of peer id), color: Option<u8 palette idx> }` (`None` = erased). Merge per cell = keep max `(lamport, author, color)` with `None < Some(_)` — the color tie-break makes a full `(lamport, author)` hash collision harmless (deterministic on every peer). This is a CRDT (commutative, associative, idempotent) under loss/dup/reorder. **No tombstone GC / no `forget()`**: max raw size is bounded (~8 MB). Untouched tiles are never sent. Revisit sparse+GC only if measurement says so. |
| D5 | Lossy repair | Draw ops travel on the gossipsub topic (possibly lossy). **Anti-entropy**: every ~2 s each peer publishes per-tile content hashes (`ReplicaSliceContentHashes`, blake3 of the tile's cells, truncated); a peer that sees a differing hash requests that tile (reliable request-response) and merges it. Cursors are lossy, latest-wins, never repaired. |
| D6 | Daily identity | Lobby `Room` gains `created_at: UnixTime` (contract change). `day = (now - created_at) / 86400`. Seed = `blake3(peer_id ‖ created_at ‖ day)`. Shape and color change live on rollover; existing pixels keep their colors. Players cannot change shape/color. **Collision resolution:** sort members by peer id; a later member whose `(shape, color)` is already taken steps to the next free combination (of 72) for that day. **Known limitation (accepted):** rooms are never deleted from the lobby and `created_at` merges by min, so a reused room name keeps its old `created_at` (day counter, seed, history key carry over) and anyone can push `created_at` back to 0. |
| D7 | Shapes / colors | Shapes: `Circle, Star, Square, Triangle, Diamond, Hexagon`. Colors: fixed palette of 12 high-contrast hues (palette index stored in cells). |
| D8 | Canvas geometry | Logical grid 1024×1024, scaled to fit window (letterboxed). Base shape radius = 16 cells. |
| D9 | Size notches | `SIZES = [0.1, 0.2, 0.35, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0]`, start index 5 (1.0). One scroll notch = ±1 index, clamped. Local per player, sent with cursor and ops. |
| D10 | Input | Left click = stamp shape (paint). Hold left = continuous stroke: stamps interpolated between previous and current mouse position, spacing `radius/2`, batched into one `Stroke` op per frame. Right click/hold = same but erase (sets cells to `None`) using the player's shape. Paint overwrites anyone's pixels. |
| D11 | Presence | Each peer streams its cursor (~30 Hz, lossy): position, size index. Others render it as that peer's shape+color outline. Side list shows members with shape/color icon. |
| D12 | Persistence | Phase 1: in-memory only + snapshot from a live member on join. Phase 2 (separate plan `PLAN_HISTORY.md`): Kademlia distributed history so late joiners get the board even if no member answers. |
| D13 | Room size target | 8 peers per room. Benchmarks at 2 and 8 in-process peers, report p50/p99. |
| D14 | Testing bar | Every plugin addition: (1) pure unit tests for reducers/merge, (2) headless in-process two(+)-`App`/two-swarm test, (3) one e2e test in the style of `tests/lobby_room_*.rs`. Starboard final acceptance: e2e where peer A paints, peer B's canvas hash converges to A's. |
| D15 | Example layout | `examples/starboard/` = shared code (as a module tree, included via `#[path]`), plus one thin bin `examples/starboard_gossip/main.rs`. `just bench-starboard` benchmarks it, writes `examples/starboard/BENCH.md`. |

---

## 2. Orientation (what exists today — verify, it may have moved)

- Crate root: `freenet-libp2p-bevy-plugin/` (this repo). Workspace tasks in `justfile`.
- `src/p2p/`: libp2p swarm. `Behaviour<T>` (`src/p2p/behaviour.rs`) already contains
  `request_response`, `peer_exchange`, `kademlia`, `identify`, `ping`, **`gossipsub`** (present
  but no public command uses it yet), `relay`, `relay_client`, `dcutr`, `mdns`.
  Public API: `p2p::Command<T>::{Net(NetCommand), Send{peer_id,payload}}`,
  `p2p::Event<T>::{Net(NetEvent), Message{from,payload}, Error}` in
  `src/p2p/__basic__/enums.rs`; Bevy resources `p2p::Commands<T>` / `p2p::Events<T>`.
  Swarm loop handlers in `src/p2p/swarm_loop/` (`dispatch_command.rs`,
  `handle_gossipsub.rs`, `handle_message.rs`, ...).
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

- [x] Add `created_at: UnixTime` to `Room` in the contract (`contract/lobby/src/...`) and
      in the plugin mirror `src/discovery/__basic__/structs.rs`.
- [x] Contract merge rule: `created_at` = **min** of both sides (deterministic,
      commutative, idempotent). Set on `Create` from the creator's `UnixTime::now()`.
- [x] Extend the CRDT-law harness (`tests/freenet_harness.rs`, `just freenet-harness`)
      so the laws cover the new field; extend `tests/contract_wire.rs`.
- [x] Expose it to apps: `JoinedRoom` (or an accessor) exposes `created_at`.
- [x] Unit test: merge of two rooms with different `created_at` keeps the min, both orders.
- [x] Rebuild wasm (`just contract`); the contract key changes — that is fine.
- [x] Full gate green (incl. contract recipes); `just e2e-join` still passes (run once).

### What was done (Step A1)

**Files changed (product):**
- `contract/lobby/src/__basic__/structs.rs` — `Room` gains `created_at: UnixTime`
  (after `room_size`, so the bincode field order matches the plugin mirror exactly).
- `contract/lobby/src/lib.rs` — `merge_room` now takes
  `existing.created_at = existing.created_at.min(incoming.created_at)`; test helper
  `room(peer, updated_at, created_at)` and a new
  `merge_keeps_min_created_at_both_orders` test.
- `src/discovery/__basic__/structs.rs` — `Room` and `JoinedRoom` each gain
  `created_at: discovery::UnixTime` (same position: after `room_size` / after `name`).
- `src/discovery/freenet/merge_room.rs` — plugin-side `min` (mirrors the contract);
  usage test now asserts `merged.created_at == min(200, 100)`.
- `src/discovery/state_machine/handle_command.rs` — `join()` derives
  `created_at = state.lobby.get(&room).map_or(now, |record| record.created_at)`.
- `src/discovery/state_machine/align_room_created_at.rs` (**new**) — a transition that
  pulls the authoritative (min) `created_at` from `state.lobby` into `state.room`, so
  `JoinedRoom` self-heals once the contract's value is read back.
- `src/discovery/state_machine/__basic__/structs.rs` — `HeartbeatTarget` carries
  `created_at`; `heartbeat_target()` fills it from the joined room.
- `src/discovery/freenet/publish_heartbeat.rs` — `created_at` parameter, written into the
  `Room` delta; `run_lobby.rs` and `connect_retry.rs` pass/carry it.
- All ~22 `Room { … }` and ~17 `JoinedRoom { … }` construction sites (fixtures in
  `src/discovery/**`, `methods/lobby/**`, `tests/`, `examples/lobby_room/**`) updated.

**Wire compatibility:** bincode is positional, so the field sits at the same index in the
contract and plugin `Room`. `tests/contract_wire.rs` now builds the plugin room with
`created_at = 7` and still round-trips through `lobby_contract::Lobby` unchanged.

**Tests added / extended:**
- Contract unit: `merge_keeps_min_created_at_both_orders` (contract/lobby/src/lib.rs).
- Plugin unit: `align_room_created_at::tests::{test_usage,
  leaves_room_untouched_without_a_record}`; min assertion in `merge_room::tests`.
- Harness: `tests/freenet_harness.rs` `one_room(updated_at, created_at)` — `gen_state`
  uses `created_at=100`, `gen_update` `created_at=200`, so the CRDT suite exercises the
  new field (`just freenet-harness`).

**Commands run (all green):**
`just build`, `just clippy`, `just fmt`, `just test` (121 passed, 3 skipped),
`just lint`, `just bevy-lint`, `just contract`, `just contract-fmt`, `just contract-clippy`,
`just contract-lint`, `just freenet-harness`, `just e2e-join` (5 peers; ring 5/5,
full mesh, converge 112 ms).

**Deviations / decisions:**
- **`created_at` derivation.** The plan said "set on `Create` from `UnixTime::now()`", but
  `handle_command` collapses `Create | Join` into one `join()`. Used one rule instead: the
  room's value if `state.lobby` already knows it, else `now()`. This also gives D6's accepted
  "reused name keeps the old `created_at`" for free, and a joiner (which cannot see a room
  without having fetched the lobby) inherits the real value.
- **No blocking wait on the min-merge.** There is no consumable "your Update was merged"
  ack (`publish_heartbeat` is fire-and-forget; `poll` only reads `GetResponse`). Convergence
  is bounded by one `LOBBY_REFRESH` (5 s): a returned lobby carrying a lower `created_at`
  is proof the min landed, and `handle_lobby` → `align_room_created_at` folds it into
  `JoinedRoom`. No new timer, no join-path fetch.

**Open issues (for later steps):**
- A room fresher than the peer's `attach` fetch can have a briefly-too-new `created_at`
  (corrected within ≤5 s). Harmless for B1 (a 86400 s day boundary), noted for awareness.
- `Created_at` can still be pushed back by a hostile client (D6, accepted).

---

## Step A2 — Room gossipsub: `Subscribe` / `Unsubscribe` / `Publish`

Goal: app can publish a payload to all peers of a room via gossipsub.

- [x] Add `NetCommand::{SubscribeRoom{room}, UnsubscribeRoom{room}}` and
      `Command::<T>::Publish{ room, payload: T }` (or a dedicated generic room message
      type — decide and document). Topic name = `"<game_id>/room/<room_name>"`.
- [x] Event: `Event::<T>::RoomMessage { room, from, payload, via: Via::Gossipsub }`
      (introduce `Via` enum now; its planned `Stream` variant was dropped — A4 removes `Via`).
- [x] Wire `handle_gossipsub.rs` + `dispatch_command.rs`. Tune gossipsub for low
      latency in small rooms (heartbeat ~100 ms, `mesh_n_low` suitable for ≤8 peers,
      `flood_publish = true`), all values in a `GossipsubConfig` struct with defaults.
- [x] Discovery integration: on `discovery::Event::Joined(room)` the plugin auto-
      subscribes; on `Left` auto-unsubscribes (configurable, default on).
- [x] Check gossipsub `max_transmit_size` (default 64 KiB) and document it; room
      payloads larger than the limit must be rejected with an error, not silently dropped.
- [x] Test-only seam: a way (`#[cfg(test)]` / dev-only feature) to inject
      `discovery::Event::Joined` + a member set into headless Apps without Freenet, so
      A5/A6/B5 headless tests need no lobby. Starboard bins and `lobby_room` must NOT use it.
- [x] Unit tests: topic name builder, config defaults.
- [x] Headless in-process test: 3 swarms on localhost (direct dial allowed — comment
      why), all subscribe, one publishes, the other two receive exactly once.
- [x] Gate green.

### What was done (Step A2)

**Files added (product):**
- `src/p2p/room_topic.rs` — `room_topic(game, room) -> identity::Topic`, the
  `"<game_id>/room/<room_name>"` builder (test covers a slash-bearing game id).
- `src/p2p/gossipsub_config.rs` — `GossipsubConfig` with `Default`
  (`heartbeat 100 ms`, `mesh_n_low 2`, `mesh_n 4`, `mesh_n_high 8`,
  `max_transmit_size 65536`, `is_flood_publish_enabled true`) and
  `config() -> Result<gossipsub::Config, ConfigBuilderError>`.
- `src/p2p/swarm_loop/room_subscriptions.rs` — `RoomSubscriptions { game, by_topic }`: registers
  `TopicHash → RoomName` on subscribe, drops it on unsubscribe, looks up inbound
  topics. Routing never parses the topic string (game ids contain `/`).
- `src/discovery/bevy_systems/follow_room.rs` — reads
  `discovery::Event` messages and pushes `p2p::NetCommand::{SubscribeRoom,
  UnsubscribeRoom}` into `p2p::PendingNetCommands`.

**Files changed (product):**
- `src/p2p/__basic__/enums.rs` — `Via::{Gossipsub,Stream}`;
  `NetCommand::{SubscribeRoom,UnsubscribeRoom}`; `Command::Publish{room,payload}`;
  `Event::RoomMessage{room,from,payload,via}`.
- `src/p2p/__basic__/structs.rs` — `RunnerConfig { mode, mdns, game, gossipsub }`.
- `src/p2p/build_swarm.rs` — takes `&GossipsubConfig`; builds the behaviour from
  `config()` (map error to `String`).
- `src/p2p/run.rs`, `src/p2p/spawn_runner.rs` — take `RunnerConfig`; `run` owns a
  `RoomSubscriptions` and threads it (plus the gossipsub limits) through dispatch/handle.
- `src/p2p/swarm_loop/{dispatch_command,dispatch_net_command,handle_swarm_event,
  handle_gossipsub}.rs` — room subscribe/unsubscribe/publish; inbound registered
  topics decode into `RoomMessage`, unknown topics still emit `NetEvent::Gossip`,
  undecodable payloads emit `Event::Error`.
- `src/p2p/mod.rs` — exports `GossipsubConfig`, `room_topic`, `RunnerConfig`, `Via`.
- `src/plugin/config.rs` — `Config` gains `game: GameId` (default `"test"`) and
  `gossipsub: GossipsubConfig`; `with_game`/`with_gossipsub` constructors.
- `methods/p2p_plugin/build_plugin.rs` — builds `RunnerConfig` from the plugin config.
- `src/discovery/__basic__/enums.rs`, `src/discovery/config.rs` — `AutoSubscribe
  { Enabled, Disabled }` (default `Enabled`), `Config::auto_subscribe`.
- `methods/plugin/build_plugin.rs` — registers `follow_room` after the
  `net_link` check (so a discovery-without-P2P app does not panic on the missing
  `PendingNetCommands`) and only when `Enabled`.
- `examples/lobby_room/main.rs` — one `game` value is passed to both the p2p and the
  discovery config.

**Public API added:** `p2p::GossipsubConfig`, `p2p::RunnerConfig`, `p2p::room_topic`,
`p2p::Via`; `NetCommand::{SubscribeRoom,UnsubscribeRoom}`, `Command::Publish`,
`Event::RoomMessage`; `plugin::Config::{game,gossipsub,with_game,with_gossipsub}`;
`discovery::AutoSubscribe`, `discovery::Config::auto_subscribe`. Breaking (0.0.1):
`build_swarm`, `run`, `spawn_runner` signatures.

**Tests added / extended:**
- `p2p::room_topic::tests::test_usage` (topic shape; `game/id` keeps its slash).
- `p2p::gossipsub_config::tests::test_usage` (defaults and the built config's getters).
- `p2p::swarm_loop::room_topics::tests::test_usage` (register / lookup / unregister).
- `p2p::swarm_loop::dispatch_command::tests::{subscribe_room_registers_its_topic,
  oversize_room_payload_is_an_error}` (70 000-byte payload → `Event::Error`).
- `p2p::swarm_loop::dispatch_net_command::tests::
  room_subscribe_and_unsubscribe_track_the_map`.
- `p2p::swarm_loop::handle_gossipsub::tests::{registered_room_topic_becomes_a_room_message,
  undecodable_room_payload_is_an_error}`; the raw-topic `test_usage` still passes.
- `discovery::bevy_systems::follow_room::tests::{test_usage,
  unrelated_events_are_ignored}` (Bevy App + `Messages<Event>` + `PendingNetCommands`).
- `p2p::run::tests::transport_only_three_swarms_receive_a_room_payload_exactly_once`
  (3 swarms, TCP, one publishes, both others receive exactly once, 1.8 s).
- `discovery::config::tests::test_usage` asserts `auto_subscribe == Enabled`.

**Commands run (all green):** `just build`, `just clippy`, `just fmt`, `just test`
(**138 passed**, 3 skipped), `just lint`, `just bevy-lint`, and `just e2e-join`
(5 real windows over the Freenet mainnet: ring 5/5, discovery 6344 ms, join 3940 ms
full mesh, converge 112 ms). No contract changes, so no contract recipes; A2 has no
e2e box, but the join e2e was run as extra evidence.

**Log evidence (real join, `.local-run/lobby_room_join-1792234085/`):** every one of
the 5 instances logged `p2p room subscribed room=room-1792234085` 33–45 ms after its
`discovery joined room` line, exactly once, with zero `p2p room unsubscribed` and zero
`p2p error` lines. The subscribe marker is a new `tracing::info!` in
`dispatch_net_command` (`p2p room subscribed` / `p2p room unsubscribed`).

**Deviations / decisions:**
- **Auto-subscribe is a Bevy system, not a state-machine transition.** The state
  machine folder is an honest boundary (no Bevy/libp2p); the bridge reads
  `discovery::Event` and writes `PendingNetCommands`. It is registered after the
  `net_link` check so a discovery-only app keeps working.
- **The game id for topics lives in `plugin::Config`.** The plan did not say where it
  comes from; `NetCommand::SubscribeRoom{room}` needs it, so `RunnerConfig.game` carries
  it into the swarm loop. The example passes one `game` to both plugins so the topic
  namespace and the lobby namespace cannot drift.
- **The test seam needs no `cfg(test)`/dev feature.** Headless injection is just the
  normal plugin boundary: write the `discovery::Status` resource (it already holds
  `room.members`) and write `discovery::Event` messages. A5/A6/B5 reuse it; starboard
  bins and `lobby_room` receive real events instead.
- **`NetEvent::Gossip` is kept** for unregistered topics (raw bytes). `NetCommand::
  {Subscribe,Publish}` also remain as the explicit-topic escape hatch, and `Publish`
  errors are now surfaced instead of ignored.
- **The 3-swarm test wires peers by direct dial** (HARD RULE §0.8 allows in-process
  transport tests). Its rationale is recorded here because `LELE031` forbids comments in
  `src/`; the test is named `transport_only_…` and settles 500 ms after full connection
  (five heartbeats) before publishing.
- **Oversize policy.** The plan says reject, not drop: `Command::Publish` pre-checks the
  serialized length against `GossipsubConfig::max_transmit_size` (65536, libp2p's own
  default) and any remaining `PublishError` is mapped to `Event::Error`.
- **`plugin::Config::new` is no longer `const`** (it builds the default `GameId`); A4 can
  revisit if a const constructor matters.

**Open issues (for later steps):**
- A publish before the gossipsub mesh knows the subscription returns
  `NoPeersSubscribedToTopic` → `Event::Error`; starboard must tolerate that until peers
  connect. A6's anti-entropy is the repair path.
- `game` is now set in two configs (`plugin::Config` + `discovery::Config`); A4's
  `RoomConfig` should unify it.
- `NetEvent::Gossip` and `Command::Publish` overlap; A4 can drop the raw variant once
  nothing needs it.
- `MIN_RING_PEERS` no longer exists on `main` (commit `87a73ce` replaced the ring gate
  with `is_lobby_attached`); the §2 orientation is stale there.

---

## Step A4 — Room API over gossipsub

Goal: apps call one room API on the only transport.

> Decision D1 revised: the room transport is **gossipsub only**. The `Broadcast` /
> `RoomTransport::Stream` design is dropped (no stable `libp2p-stream`; see §1 D1). Step
> A3 was deleted. The reliability split (`Room { reliability: Lossy | Reliable }`) was
> dropped as well; reliable transfers stay internal to A5/A6. Room-member tracking moved
> to **A5**, whose snapshot request is its first consumer.

- [x] `plugin::Config` gains `room: RoomConfig { anti_entropy: Option<AntiEntropyConfig>,
      gossipsub: GossipsubConfig }` with `Default` (anti-entropy on). Keep `Config::new`
      working (builder methods for new fields).
- [x] `Command::<T>::Publish{ payload }` publishes to the currently joined room's topic;
      `Command::<T>::PublishTo{ room, payload }` names an explicit room. (`Room{payload}`
      was renamed: it named a noun, not the action.)
- [x] Remove the now-single-variant `Via` enum and the `via` field on
      `Event::RoomMessage`; update `handle_gossipsub` and the A2 tests.
- [x] Unit tests for config defaults/builders, the joined-room tracking and the
      no-joined-room error; the 3-swarm headless test now publishes via `Command::Publish`.
- [x] Update `README.md` "Using the plugin" with the room API + defaults.
- [x] Gate green.

### What was done (Step A4)

**Files added (product):** `src/p2p/room_config.rs` (`RoomConfig`),
`src/p2p/anti_entropy_config.rs` (`AntiEntropyConfig { interval, max_parts_in_flight }`,
default 2 s / 8).

**Files changed (product):** `src/p2p/mod.rs` (exports `RoomConfig`/`AntiEntropyConfig`,
drops `Via`); `src/p2p/__basic__/structs.rs` (`RunnerConfig.gossipsub` → `room`);
`src/p2p/__basic__/enums.rs` (`Via` deleted, `RoomMessage.via` dropped,
`Command::{Publish{payload}, PublishTo{room,payload}}`); `src/p2p/swarm_loop/room_topics.rs`
(`RoomSubscriptions.joined`, set on `register`, cleared on `unregister`);
`src/p2p/swarm_loop/dispatch_command.rs` (`publish_joined` helper, "no joined room" error);
`src/p2p/swarm_loop/handle_gossipsub.rs`, `src/p2p/run.rs`, `src/p2p/spawn_runner.rs`,
`src/plugin/config.rs` (`room` field + `with_room`), `methods/p2p_plugin/build_plugin.rs`,
`README.md`.

**Public API added:** `p2p::{RoomConfig, AntiEntropyConfig}`;
`Command::{Publish, PublishTo}`; `plugin::Config::{room, with_room}`. Removed:
`p2p::Via`, `Event::RoomMessage.via`, `plugin::Config::{gossipsub, with_gossipsub}`,
`Command::Publish{room,..}` (breaking, 0.0.1).

**Tests:** `p2p::room_config`, `p2p::anti_entropy_config`, `room_topics::tests::test_usage`
(joined set/clear), `dispatch_command::tests::{subscribe_room_registers_its_topic_and_joins_it,
publish_without_a_joined_room_is_an_error, oversize_room_payload_is_an_error}`;
`run::tests::transport_only_…` updated to `Command::Publish{payload}`.

**Commands run (all green):** `just build`, `just clippy`, `just fmt`, `just test`
(**141 passed**, 3 skipped), `just lint`, `just bevy-lint`.

**Deviations / decisions:**
- **`Publish` / `PublishTo` instead of `Room`** (user call): the bare verb is the common
  case (joined room), the qualified one names a room.
- **No reliability split**: reliable transfers are internal to A5/A6, not app-facing.
- **Member tracking deferred to A5**: A4 has no product consumer for it (write-only state).

**Open issues (for later steps):**
- `RoomSubscriptions.joined` is a single slot: a later `SubscribeRoom` replaces it. One room per
  session is the current model.

---

## Step A5 — Generic room state sync: Lamport clock + snapshot on join

Goal: reusable plumbing so any app can keep a replicated CRDT state per room.

> Decision (A5 design): `PendingOps<S>` ships ops on its **own raw gossip topic**
> `<game>/room/<room>/sync` (app's `T` stays free on the room topic); snapshots go over a
> **new request/response behaviour** (`/<game>/room-sync/1`) with distinct request/response
> frames. The member set comes from `discovery::Status`, not from the swarm loop.

- [x] `RoomState` trait (`Op`/`Snapshot`, `apply`/`snapshot`/`merge`) and a
      `SyncPlugin<S>` owning `Resource<S>` and a `LamportClock` resource (`tick`,
      `observe`).
- [x] On `Joined`: `Fetch{part: 0}` to the **lowest connected member** (self excluded), retry
      the next on `SnapshotMissing`/`Tick`. Snapshots split into `PART_BYTES` (48 KiB) parts,
      one response per part; ops received before the snapshot are buffered and applied after
      `merge`.
- [x] App ops go through the `PendingOps<S>` writer; the plugin wraps them with lamport +
      author and publishes them on the room's sync topic.
- [x] Unit tests: Lamport laws; buffering-before-snapshot reducer; part split/join round-trip;
      frame codec; sync topic; peer author.
- [x] Headless **2-`App`** late-join convergence test: two in-process `App`s wired by direct
      dial (§0.8 allows in-process transport tests); the incumbent holds `7 → 42`, the joiner
      joins late and must receive a `SnapshotPart` and converge to the same `Map`.
- [x] Gate green.

### What was done (Step A5)

**p2p surface (new):** `SnapshotRequest`/`SnapshotResponse` (`Fetch`; `Part`/`Missing`, in
`src/p2p/__basic__/enums.rs`), `p2p::sync_protocol(&GameId)` (`/<game>/room-sync/2`), a third
`request_response` behaviour `room_sync`, `RoomSubscriptions` + `PendingReplies`,
`handle_room_sync`, and the `NetCommand::{Unsubscribe,
FetchSnapshot, RespondSnapshot}` / `NetEvent::{SnapshotRequest, SnapshotPart, SnapshotMissing}`
variants. The responder cannot compute `S::Snapshot` in the swarm loop, so the loop holds the
`ResponseChannel` under a `u64` id and the plugin answers later via `RespondSnapshot`.
`NetEventChannel` became a **multi-subscriber broadcast bus** (`subscribe()`), so discovery and
`room_sync` both consume `NetEvent`s; `discovery::libp2p::NetLink.events` and
`discovery/run.rs` follow.

**`room_sync` domain (new):** `RoomState`, `OpMeta`, `LamportClock`, `peer_author`,
`SyncFrame`, `sync_topic`, `split_parts`/`join_parts`, `PendingOps<S>` writer,
`SyncPlugin<S>` (+ `methods/sync_plugin/build_plugin.rs`), and a pure `state_machine`
(`State`, `Input`, `Output`, `Phase`, `update`, `fetch_next`, `handle_*`, `tick`) with Bevy
systems `drive` (inbound) and `write_ops` (outbound).

**Public API added:** `room_sync::{RoomState, OpMeta, LamportClock, peer_author, SyncFrame,
sync_topic, split_parts, join_parts, PendingOps, SyncPlugin, SyncState, NetEvents}`;
`p2p::{SnapshotRequest, SnapshotResponse, sync_protocol}`; `NetCommand::{Unsubscribe, FetchSnapshot,
RespondSnapshot}`; `NetEvent::{SnapshotRequest, SnapshotPart, SnapshotMissing,
SnapshotFailed}`.

**Fetch failure is retried.** A `Fetch` that races the dial produced `OutboundFailure`, which
was ignored, leaving the joiner stuck. `handle_swarm_event` now maps `RoomSync(OutboundFailure)`
to `NetEvent::SnapshotFailed`, `drive` feeds it as `Input::Failed`, and `retry_fetch` re-asks
the **same** source for the first missing part, falls back to the next un-asked member, and only
then gives up (draining buffered ops). Unit test:
`room_sync::state_machine::update::tests::retries_the_same_source_for_the_missing_part`.

**Commands run (all green):** `just build`, `just clippy`, `just fmt`, `just test`
(**155 passed**, 3 skipped), `just lint`, `just bevy-lint`.

**Deviations / decisions:**
- **Removed the swarm-loop member map.** `NetCommand::SetRoomMembers`, `RoomMembers` and
  `sync_room_members` were dropped: the reducer reads members from `discovery::Status`, so
  the loop-side map was write-only state. The plugin is discovery-*optional*
  (`Option<Res<Snapshot>>`, `Option<MessageReader<Event>>`).
- **Snapshot parts recompute per `Fetch{part}`**, split by the `PART_BYTES` constant (48 KiB)
  rather than a `RoomSyncConfig` struct; `merge` is idempotent so a mid-fetch re-split is
  harmless.
- The plugin type is `room_sync::SyncPlugin` (not `RoomSyncPlugin`): LELE012 forbids a type
  repeating its module name, and a second `Plugin` would collide with discovery's
  `#[atomic_delegates]` path (`methods/plugin/`).
- `p2p::sync_protocol` (not `room_sync_protocol`): the old name was parsed as a `RoomSync`
  method file.

**Open issues (for later steps):**
- A real-lobby convergence e2e (two examples meeting via Freenet, asserting equal hashes) is
  still **A7** (`tests/room_messaging.rs`, `just e2e-room`) — the HARD RULE requires lobby
  discovery in examples, so it is a separate, X11/Freenet step.

---

## Step A6 — Tiled anti-entropy

Goal: heal divergence caused by lossy delivery.

> **A6 vocabulary (agreed with the user — do not re-litigate).** Inner-workings types get
> descriptive multi-word names that say what they hold; one-word names are reserved for
> user-facing concepts (Room, Lobby). Banned words: "shard", "region", "partition",
> "digest", "fingerprint".
>
> - **Replica slice** = one independent piece of the replicated room state that is
>   compared and repaired on its own (starboard: one 32×32-cell tile of the canvas).
> - **`ReplicaSliceIndex(u32)`** — *which* slice: its position in the replica's fixed list
>   of slices (starboard: tile `y * 32 + x`, 0..1024). Never changes. One concrete type:
>   no associated key type and no bytes twin.
> - **`ReplicaSliceContentHash(u64)`** — *what is in* a slice right now: blake3 of the
>   slice's content, truncated to 64 bits. Changes on every edit. Same hash on two peers ⇒
>   same content ⇒ nothing to send.
> - **`ReplicaSliceContentHashes(Vec<(ReplicaSliceIndex, ReplicaSliceContentHash)>)`** —
>   one peer's hash for every non-empty slice, sorted by index; what it announces each
>   interval. Replaces the raw `Vec<(Vec<u8>, u64)>`.
> - **`ReplicaSyncTopicMessage<Op>`** (renames `SyncFrame`) — one message published to all
>   room members on the `/sync` gossip topic. Each variant tells the receiver its job:
>   `ApplyThisOp{room,meta,op}` (an op the sender just applied) and
>   `CompareSliceContentHashes{room,hashes: ReplicaSliceContentHashes}` (the sender's
>   current per-slice hashes; fetch any slice that differs).
> - **`SnapshotRequest::FetchReplicaSlice{room,index,part}`** — reliable request to *one*
>   peer for the current content of *one* slice, sent after a hash mismatch. Reuses the
>   `SnapshotResponse::Part` frames (`part` stays the 48 KiB byte-chunk index).
> - **`State.slice_fetches_awaiting_reply`** — slices already requested from each peer
>   whose reply has not arrived; caps duplicate/concurrent requests.
> - **Generics renamed first** (separate `refactor!:` commit): `S` → `Replica`,
>   `O` → `Op`, `T` (p2p `Command`/`Event`) → `Payload`.
> - Slice hashes travel on the **`/sync` topic, not the app room topic** (A5 keeps the
>   app's `Payload` alone on the room topic).

**Commit 1 — `refactor!:` vocabulary (no behaviour change)**

- [x] Rename generics: `S` → `Replica` (`SyncPlugin`, `PendingOps`, `write_ops`, …),
      `O` → `Op` (`state_machine::{State, Input, Output}`, `handle_*`), `T` → `Payload`
      (`p2p::{Command, Event, Commands, Events}` and their users).
- [x] `SyncFrame` → `ReplicaSyncTopicMessage` enum with only `ApplyThisOp{room,meta,op}`.
- [x] ~~`AntiEntropyConfig.max_parts_in_flight` → `max_slice_fetches_per_peer`.~~ Moot:
      `d669d7d` deleted `AntiEntropyConfig`/`RoomConfig` as unused; commit 2 reintroduces it.
- [x] Gate green; commit.

**Commit 2 — `feat(room_sync):` anti-entropy over replica slices**

- [ ] Types in `src/room_sync/` (one file each): `ReplicaSliceIndex(u32)`,
      `ReplicaSliceContentHash(u64)`, `ReplicaSliceContentHashes(Vec<(ReplicaSliceIndex,
      ReplicaSliceContentHash)>)` with
      `slices_differing_from(&self, remote: &Self) -> Vec<ReplicaSliceIndex>` — indices whose
      hash in `remote` differs from, or is missing in, `self`.
- [ ] Extend `RoomState`:
      `fn slice_content_hashes(&self) -> ReplicaSliceContentHashes;`
      `fn slice_snapshot(&self, index: ReplicaSliceIndex) -> Self::Snapshot;`
      (single-slice test states return only `ReplicaSliceIndex(0)`).
- [ ] Wire:
      - `ReplicaSyncTopicMessage::CompareSliceContentHashes{room, hashes}` on `/sync`.
      - `SnapshotRequest::FetchReplicaSlice{room, index, part}`; the swarm loop parks the
        `ResponseChannel` and the plugin answers via `RespondSnapshot`, exactly as A5's
        `Fetch`. Replies reuse `SnapshotResponse::Part`.
- [ ] Reducer (`state_machine/`, transitions return outputs):
      - `Input::AntiEntropyTimerFired{local}` → `Output::PublishSliceContentHashes{room, hashes}`.
      - `Input::RemoteSliceContentHashes{from, local, remote}` →
        `Output::FetchReplicaSlice{peer, room, index}` for each
        `local.slices_differing_from(&remote)` not already awaited.
      - `Input::ReplicaSliceMerged{from, index}` and `Input::Failed` clear the entry.
      - `State.slice_fetches_awaiting_reply: BTreeMap<PeerId, BTreeSet<ReplicaSliceIndex>>`,
        at most `AntiEntropyConfig.max_slice_fetches_per_peer` (default 8) per peer.
      - Acts only in `Phase::Ready`; ignored otherwise.
- [ ] Bevy:
      - `publish_slice_content_hashes<Replica>`: `Timer(AntiEntropyConfig.interval)`
        (default 2 s); registered only when `RoomConfig.anti_entropy` is `Some`.
      - `drive`: decodes `CompareSliceContentHashes`, reads `Res<Replica>` and feeds the
        local `ReplicaSliceContentHashes` alongside, so the reducer stays Bevy-free;
        answers `FetchReplicaSlice` with `slice_snapshot(index)`.
- [ ] Unit tests: `slices_differing_from` (equal / missing / extra / differing); per-peer cap;
      no duplicate fetch while awaited; failure clears the entry; not-`Ready` ignored;
      `ReplicaSyncTopicMessage` bincode round-trip.
- [ ] Headless test `transport_only_three_apps_converge_after_dropping_30_percent_of_ops`:
      3 Apps (direct dial, §0.8), a toy state split into slices, a seeded 30 % drop of
      inbound `ApplyThisOp` (test-only seam at the plugin boundary, no cargo feature);
      after traffic stops, all peers' `slice_content_hashes()` are equal within 3 intervals.
- [ ] Gate green; commit with the PLAN.md update.

### What was done (Step A6)
_(agent fills)_

---

## Step A7 — Plugin e2e: room messaging over real discovery

- [ ] New `tests/room_messaging.rs` (`#[ignore]`, feature `e2e`, registered in
      `Cargo.toml` like the existing ones) reusing `tests/common/`: 3 `lobby_room`-style
      processes meet via Freenet lobby (HARD RULE), join one room, each sends a room
      message, all receive. Add `just e2e-room` recipe.
- [ ] Extend `examples/lobby_room` minimally if needed (e.g. a `--room-ping` flag that
      logs received room messages for `tests/common/log_parse.rs`).
- [ ] Run it once; record result.

### What was done (Step A7)
_(agent fills)_

---

## Step A8 — Join self-heal gate + `JoiningRoom` screen

Depends only on A1. A join becomes a three-phase lifecycle instead of a boolean: the room
enters `RoomPhase::Joining`, the plugin waits until the contract's `created_at` has been
observed for the room (the A1 min-merge landing), then flips to `RoomPhase::Ready`. The
default UI shows a black "Joining room <name>" screen and blocks all input until ready.

- [x] `RoomPhase { Joining, Ready }`; `JoinedRoom.phase`; `State.joining_since`.
- [x] `wait_for_contract_self_heal` transition: ready when the lobby record's
      `created_at` equals the room's; `JOIN_HEAL_TIMEOUT = 30s` fallback (warn + proceed).
      Called from `handle_lobby` (after `align_room_created_at`) and `tick`.
- [x] `Event::RoomReady(room)`.
- [x] `Screen::JoiningRoom` + `spawn_joining_room` (black screen + "Joining room <name>");
      `pick_screen` maps `Joining → JoiningRoom`, `Ready → InRoom`.
- [x] Input blocked structurally (no interactive entities; all input systems stay gated to
      `Lobby`/`InRoom`).
- [x] Tests: `wait_for_contract_self_heal` (wait / heal / timeout / idle), `pick_screen`
      (Joining vs Ready), `spawn_joining_room` + PNG preview; `just e2e-join` re-run green.

### What was done (Step A8)

**Files (new):** `src/discovery/state_machine/wait_for_contract_self_heal.rs`,
`src/discovery/ui/bevy_systems/spawn_joining_room.rs`.
**Files (changed):** `src/discovery/__basic__/enums.rs` (`RoomPhase`),
`src/discovery/__basic__/messages.rs` (`Event::RoomReady`), `src/discovery/__basic__/structs.rs`
(`JoinedRoom.phase`), `src/discovery/state_machine/state.rs` (`joining_since`),
`handle_command.rs`, `handle_lobby.rs`, `tick.rs`, `state_machine/mod.rs`,
`state_machine/__basic__/constants.rs` (`JOIN_HEAL_TIMEOUT`), `ui/__basic__/states.rs`
(`Screen::JoiningRoom`), `ui/__basic__/components.rs` (`JoiningRoomRoot`),
`ui/bevy_systems/pick_screen.rs` + `mod.rs`, `ui/mod.rs`,
`methods/default_ui_plugin/build_plugin.rs`, `examples/lobby_room/overlay.rs`
(`sync_visibility` gates on `phase == Ready`), plus all 22 `JoinedRoom { .. }` fixture
sites (`phase: Ready` where a test expects `InRoom`).

**Behaviour.** Readiness = `state.lobby[room].created_at == state.room.created_at`
(post-`align_room_created_at`); the list-join path is already observed, so it heals on the
first check, while the **create** path shows the black screen until the deploy + read-back
populates the lobby (≤ `JOIN_HEAL_TIMEOUT`). `Event::Joined` still fires at join time, so
the `lobby joined room=<r>` e2e oracle and all join/converge checks are unchanged; the
libp2p mesh is **not** gated on readiness — only the UI is.

**Commands run (all green):** `just build`, `clippy`, `fmt`, `test` (**126 passed**, 3
skipped), `lint`, `bevy-lint`, `contract` recipes (contract untouched), `just e2e-join`
(5/5 ring, full mesh, converge 99 ms).

**Open issues:** the create path's `JoiningRoom` is not covered by an automated e2e (the
discovery e2e uses `--ui custom`, the join e2e joins from the list); a first `e2e-join`
attempt failed on a pre-existing freenet ring flake (a peer stuck at 2 ring peers <
`MIN_RING_PEERS`), unrelated to this step — the re-run was green. Revisit in A7/B7 when
the room-messaging/starboard e2e lands.

---

# PART B — Starboard example game

Layout (D15):

```
examples/starboard/            # shared module tree, NOT itself an [[example]]
  mod.rs                       # pub mod canvas; identity; input; render; sync; ui; app;
  PLAN.md  PLAN_HISTORY.md  BENCH.md
  canvas/  identity/  input/  render/  sync/  ui/  app/
examples/starboard_gossip/main.rs   # #[path = "../starboard/mod.rs"] mod starboard;
```

Register it in `Cargo.toml` as an `[[example]]` with `required-features = ["e2e"]`.
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
      `(TileId, Tile)`, one `ReplicaSliceIndex` + `ReplicaSliceContentHash` (blake3) per tile).
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
- [ ] Register `SyncPlugin<Canvas>`; local ops go through it; remote ops applied
      and dirty tiles re-uploaded.
- [ ] Presence: cursor message `{ pos, size_idx }` at ~30 Hz lossy on the gossipsub topic;
      render other peers' cursors as their shape+color outline; side panel with members
      (shape icon + color + display name). Remove cursors of peers that left.
- [ ] Daily rollover recomputed every second from `created_at`.
- [ ] Bin `starboard_gossip` + `just run-starboard`.
- [ ] Headless in-process test: 2 Apps, A paints scripted strokes, B canvas hash equals A's.

### What was done (Step B5)
_(agent fills)_

---

## Step B6 — `just bench-starboard`

- [ ] Headless benchmark binary/test (no window): N ∈ {2, 8} in-process peers, scripted
      strokes at 60 ops/s per peer for 20 s.
- [ ] Measure: op latency p50/p99 (send→apply), bytes/s per peer, % ops dropped before
      anti-entropy, time to canvas-hash convergence after stop.
- [ ] Write a table to `examples/starboard/BENCH.md` (overwrite each run, include date,
      machine info).

### What was done (Step B6)
_(agent fills, include table summary)_

---

## Step B7 — Starboard e2e acceptance

- [ ] `tests/starboard_e2e.rs` (`#[ignore]`, `e2e`), reusing `tests/common/`: 3 real
      `starboard_gossip` windows meet via Freenet lobby only (HARD RULE), join one
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
      (lobby → room → gossipsub → room_sync → Canvas).
- [ ] Root `README.md`: mention starboard and room API defaults.
- [ ] Revisit open issues collected in previous "What was done" sections; list
      remaining ones here.

### What was done (Step B8)
_(agent fills)_

---

# PART C — Distributed history (later phase)

See **`examples/starboard/PLAN_HISTORY.md`**. Start it only after Part B is complete.
