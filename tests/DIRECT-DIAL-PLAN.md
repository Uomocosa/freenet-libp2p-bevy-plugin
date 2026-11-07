# Direct dial plan — friends, known peers, fast cross-OS

## Instruction to the agent

Perform this plan and nothing else. Act as orchestrator (`opencode-orchestrator` skill): the work
is done by `opencode run` workers, never by Claude subagents. Verify every result yourself and do
not trust worker reports: run the gates, read the diff, and check each test proves what it claims.

Commit and push verified work on `main`, in as many commits as make sense, with hooks on.

**Stop only when the plan is finished, or for something you cannot decide yourself**, such as a
real product bug or a choice only the user can make. New public names are not a reason to stop:
choose names that follow the rules and list them in the final report.

## Goal

Peers that already know each other connect quickly by direct dial. Mainnet stays the only proof of
connecting with no prior knowledge. Cross-OS runs become much faster.

## Read first

`tests/E2E-LAYERS-PLAN.md`, `tests/CROSS-OS-PLAN.md`, `tests/README.md`, `.github/workflows/e2e-cross.yml`.
Skills: `lele-rs`, `lele-syntax-rs` (with RATIONALE.md and SMELLS.md), `lele-lint-rules-rs`, `libp2p`, `bevy-rs`.

## Decisions (agreed with the user)

| # | Topic | Decision |
|---|---|---|
| D1 | HARD RULE | The ban on known addresses in `tests/README.md` applies to the mainnet discovery tests. Direct-dial tests may use known addresses. Update the README to say so. |
| D2 | Friend connected | Connected and visible only. "Join a friend's room" is out of scope. |
| D3 | Friends tab | Lives in the plugin's default UI, so every game gets it. |
| D4 | Sharing addresses in tests | No fixed port or LAN IP input. Peers listen on any free port and log their real addresses; `Orchestrator` reads them and passes them on, across machines through the board. |
| D5 | Public addresses | LAN addresses may appear on the shared board, even though the repo is public. Never publish keys. |
| D6 | One type | `identity::Peer` (id plus addresses) is the only peer type in the dial code. There is no separate address type. |
| D7 | Two kinds, one list | One state machine with one list of `DialKnownPeer`; friends and previously connected peers differ only by kind and retry rules. |
| D8 | Plugin setup | The real setup stays visible in each example's `main()`. |
| D9 | Windows PC | Old and low-end: never more than 1 worker there. |
| D10 | `time_scale` | Stays 1. Do not raise it to make a run pass. |

## Code shape

Shape, not final text: adjust where `lele-lint` requires it.

```rust
// Identity: without with_keypair, a new key is generated as today.
let identity = identity::SavedKeypair::load_or_create(&args.identity_file)?;
plugin::Config::<CursorUpdate>::new(transport, p2p::MdnsMode::Disabled)
    .with_game(game.clone())
    .with_keypair(identity);

// The text box parses with identity::Peer::try_from(text); a bad address shows an error beside it.

// src/direct_dial/state_machine/: no I/O and no clock inside.
pub enum KnownPeerKind { Friend, PreviouslyConnected }

pub struct DialKnownPeer {
    pub peer: identity::Peer,
    pub kind: KnownPeerKind,
    pub failed_attempts: u32,
    pub next_attempt_at: discovery::UnixTime,
}

pub enum Input {
    Tick,
    FriendAdded(identity::Peer),
    FriendRemoved(identity::Peer),
    PeerConnected(identity::Peer),
    DialFailed(identity::Peer),
}

pub enum Output {
    Dial(identity::Peer),
    SavePeerBook(direct_dial::PeerBook),
}

impl DialState {
    pub fn update(&mut self, input: Input, now: discovery::UnixTime) -> Vec<Output> {}
}

// libp2p's failure event carries only the id: the thin adapter looks up the full Peer.

// The app adds one plugin beside the existing ones. It only sends p2p::Libp2pCommand::Dial.
app.add_plugins(direct_dial::Plugin::new(direct_dial::Config {
    peer_book_file: args.peer_book_file.clone(),
}));

// src/direct_dial/ui/
pub struct FriendsTab;
pub struct FriendAddressInput;
pub struct AddFriendButton;
pub struct FriendRow { pub peer: identity::PeerId }

// src/testing/
#[async_trait]
pub trait CheckpointBoard: Send + Sync {
    async fn mark_completed(&self, checkpoint: &Checkpoint) -> Result<(), testing::Error>;
    async fn has_completed(&self, checkpoint: &Checkpoint) -> Result<bool, testing::Error>;
    async fn publish_peer(&self, name: &str, peer: &identity::Peer) -> Result<(), testing::Error>;
    async fn find_peer(&self, name: &str) -> Result<Option<identity::Peer>, testing::Error>;
}

// tests/e2e/friends_reconnect.rs: a test may pin its network.
#[e2e_test(game = "starboard", network = "direct-dial")]
fn friends_reconnect(test: &mut Orchestrator) {
    test.peer("alice").runs([
        add_friend("bob"),
        wait_until_friend_is_connected("bob"),
        exit_app(),
    ]);
    test.peer("bob").runs([
        wait_until_friend_is_connected("alice"),
        stay(),
    ]);
    test.expect_no_freenet_node();
}
```

`network = "direct-dial"` means real libp2p with no Freenet node: `FakeNetworkPlugin` would prove
nothing about dialing. `add_friend("bob")` waits until bob's address is known, then tries all of
his addresses. Retry rules: friends keep retrying with capped backoff; previously connected peers
retry a few times per run, and entries that stay dead are dropped.

## Phases

Each phase leaves `just clippy`, `just lint`, `just bevy-lint`, `just fmt` and `just test` green.

- [x] **0. Fix the cross-OS click.** Run `42386802626` failed because the two Linux windows were stacked (the user saw it), so `l2`'s click never landed. Tile them in `e2e-cross.yml`, or otherwise stop them overlapping. Re-run S1 with `time_scale=1`, reusing the bundle with `bundle_run_id` if the game did not change. Done when `check` passes on `lan`. Also confirm both videos reach Telegram.
- [x] **1. Research, no code.** Read the current rust-libp2p docs on dialing known addresses, retry and backoff, storing peer addresses, and any peer-store support. Follow what libp2p suggests where it fits this shape, and record the findings in this file.
- [x] **2. Persistent identity.** `SavedKeypair`, `.with_keypair`, `--identity-file`. `P2PPlugin` currently generates a new keypair every launch, so nothing below can work first. Each test peer gets its own identity. Keys never go into artifacts.
- [x] **3. Friends.** `Peer` parsing, Friends tab, add and remove, saved to disk, connected or offline per row.
- [x] **4. Known peers and dialing.** `DialState` with unit tests for backoff and dropping, the peer book, the adapter and `direct_dial::Plugin`. It must not block or replace Freenet discovery.
- [x] **5. Tests.** Add the `direct-dial` network, per-test network pinning, and address passing through `Orchestrator` on one machine. Add the tests in the table below.
- [ ] **6. Fast cross-OS.** Minimal version only: `CheckpointBoard`, `LocalCheckpointBoard`, `GitHubCheckpointBoard` in `tests/e2e/`, machine placement (`E2E_MACHINE`, `E2E_PLACEMENT`), `E2E_ROLE=check`, and the test program in the bundle. Try commit statuses first (`statuses: write`); descriptions are short (about 140 characters), so verify several addresses fit, and otherwise pick another GitHub mechanism. Poll about every 10 seconds, only while waiting. Routine cross-OS runs use direct dial, and the mainnet run stays as a separate proof.
- [x] **7. Docs.** Update `tests/README.md`, `tests/CROSS-OS-PLAN.md` (L12 and "no cross-machine control channel" change: direct-dial tests may share addresses; mainnet discovery tests may not) and `tests/E2E-LAYERS-PLAN.md`.

## Phase 1 findings

Research against rust-libp2p 0.57.1 (`Cargo.lock`: `libp2p 0.57.1`, `libp2p-swarm 0.48.0`, `libp2p-identity 0.3.1`, `libp2p-core 0.44.0`).

1. Dial a known peer: `DialOpts::peer_id(id).condition(pc).addresses(vec).build()`, then `swarm.dial(opts)`.
   - Sources: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libp2p-swarm-0.48.0/src/dial_opts.rs` (`DialOpts` struct, `peer_id`, `addresses`, `build`); same file `PeerCondition`: `Disconnected`, `NotDialing`, `DisconnectedAndNotDialing` (default), `Always`; `libp2p-swarm-0.48.0/src/lib.rs` `Swarm::dial` (checks condition against pool, dedups addrs, drops listened addrs, `NoAddresses` if empty).
   - Multiple addresses are tried concurrently per dial: pool default concurrency factor 8, overridable per dial via `override_dial_concurrency_factor`; without `.extend_addresses_through_behaviour()`, behaviour-provided addresses are discarded. Sources: `libp2p-swarm-0.48.0/src/connection/pool.rs` (`dial_concurrency_factor`, `smart_dial`, `ConcurrentDial`); `dial_opts.rs` (`override_dial_concurrency_factor`, `extend_addresses_through_behaviour`); repo override is 1 (sequential): `methods/swarm_loop_state/dial_peer.rs:17-21`, `src/p2p/__basic__/constants.rs:5`.
   - Fits the plan.

2. Failure events: `SwarmEvent::OutgoingConnectionError { connection_id, peer_id: Option<PeerId>, error: DialError }`; `peer_id` is NOT always present (`None` for address-only/unknown-peer dials, via `DialOpts::get_peer_id`). `DialError` variants: `LocalPeerId`, `NoAddresses`, `DialPeerConditionFalse`, `Aborted`, `WrongPeerId`, `Denied`, `Transport`. Sync `swarm.dial` errors return the same `DialError` immediately (and also notify the behaviour).
   - Sources: `libp2p-swarm-0.48.0/src/lib.rs` (`OutgoingConnectionError` ~line 228, `DialError` ~line 1532, `Swarm::dial` ~line 428); repo mapping `src/p2p/swarm_loop_state.rs:414-416,473-487` (`Some` -> `Libp2pEvent::DialFailed { peer_id, reason }`, `None` -> debug log only), `src/p2p/__basic__/enums.rs:19-22`.
   - Fits the plan (adapter looks up the full `Peer` by id; `DialState` always dials known peers so `peer_id` is `Some`; `None` case can only log).

3. No built-in retry or backoff for dials in `libp2p-swarm 0.48.0` (source search for `backoff`/`retry` finds nothing; each `Swarm::dial` is one-shot ending in `ConnectionEstablished` or `OutgoingConnectionError`). Upstream-suggested retry policy: `unverified` (no explicit guidance found in the sources read); the shape implies the caller re-issues `dial`, which is what the plan's `Tick`-driven `DialState` already does.
   - Sources: `libp2p-swarm-0.48.0/src/` (absence); `Swarm::dial` docs in `src/lib.rs`.
   - Fits the plan (backoff lives in `DialState`, not libp2p).

4. A peer store exists (`libp2p-peer-store 0.1.0`, `Behaviour` + `Store` trait + in-memory `MemoryStore` with LRU-bounded, frequency-ordered addresses), is compatible with 0.57 (`libp2p-0.57.1/Cargo.toml` optional dep `peer-store`, `libp2p-peer-store 0.1.0` depends on `libp2p-swarm 0.48.0`), is NOT currently used (absent from `Cargo.lock`, not in our `Cargo.toml` features), and does NOT persist to disk (in-memory only; `Store` trait has no persistence methods; docs: "PeerStore is local/standalone", must be hooked up manually, dials need `extend_addresses_through_behaviour`).
   - Sources: `https://docs.rs/libp2p-peer-store/latest/libp2p_peer_store/` (Usage, Important Discrepancies), `.../memory_store/`, `.../trait.Store.html`; `libp2p-0.57.1/Cargo.toml` lines 71, 115, 212; `Cargo.toml:101` (our features).
   - Fits the plan (peer-store is an optional in-memory address cache; the plan's file-backed `PeerBook` is still needed for cross-run persistence).

5. Persistent identity: `Keypair::to_protobuf_encoding` / `Keypair::from_protobuf_encoding` (`libp2p-identity-0.3.1/src/keypair.rs`); `SwarmBuilder::with_existing_identity(keypair)` (`libp2p-0.57.1/src/builder/phase/identity.rs:13-21`), `with_new_identity()` generates fresh.
   - Sources: as above; repo already builds with `with_existing_identity` (`methods/runner_config/build_swarm.rs:15`).
   - Fits the plan (`SavedKeypair::load_or_create` + `Config::with_keypair` are new code on top of existing `with_existing_identity`).

6. Repo today: swarm is built in `methods/runner_config/build_swarm.rs` (already `with_existing_identity`, but the key comes from a fresh `Keypair::generate_ed25519()` in `methods/p2p_plugin/build_plugin.rs:38`); `plugin::Config` (`src/plugin/config.rs`) has no `with_keypair`. `Libp2pCommand::Dial(identity::Peer)` flows `dispatch_libp2p_command` (`src/p2p/swarm_loop_state.rs:115-119`) -> `dial_peer` (`methods/swarm_loop_state/dial_peer.rs`: parses addrs, silently skips empty/unparseable sets, `DisconnectedAndNotDialing`, concurrency 1; unparsable peer id falls back to address-only dials; sync `dial` errors become `DialFailed`).
   - What must change for the `DialState` adapter: new `direct_dial` module (`DialState` sans-IO, `PeerBook` load/save, adapter mapping `DialFailed`/`PeerConnected` by peer id to `Input::DialFailed`/connected and emitting `Libp2pCommand::Dial`), `SavedKeypair` + `Config::with_keypair`, Friends tab UI, test `CheckpointBoard` hooks. Nothing in the existing dial path needs to change.
   - Fits the plan.

## Cross-OS baseline (before Phase 6)

S1 over mainnet discovery on `lan`, `time_scale=1`:

- Build: `build-windows` 27 min, `build-linux` under 1 min with a warm cache (run `42386802626`).
- Run with reused bundles: peers about 65 s, whole workflow 107 s (run `42399468572`).

## Tests we want

| Test | Network | Proves |
|---|---|---|
| Connection with no prior knowledge | Mainnet only | Freenet discovery |
| Direct dial | Real libp2p, localhost or LAN | Dialing an address works |
| Friends | Real libp2p | Add a friend, reconnect, restart |
| Known peers | Real libp2p | Reconnect on a later run with no re-adding |

Keep the existing no-network tests for UI review.

## Rules for every worker

- No `#[allow]` or `#[expect]` for pedantic or nursery lints, no lint, clippy or lele config changes, no weakened or deleted tests.
- Never pipe `just` through `tail`, `head` or `grep`; redirect to a file.
- Mainnet: 90 seconds between runs, Telegram silent unless the run is meant to send, one retry only for plain network slowness.
- Workers make no git writes.

## Lessons from the last run

- A worker's "PASS" is not proof. Check that each test still proves what its scenario claims: S4c once passed after being weakened into a copy of S4b.
- The fake network hides timing bugs: always confirm on real libp2p or mainnet.
- `tokio::select!` over multi-step socket reads is not cancel safe. Read in a dedicated task.
- Do not sync peers with `sleep`, and never wait on a member count after a peer may have left.
- `opencode run` can exit while the session continues: use the session export, and wait until it is idle with no build running.
- The harness's `cleanup_stale` kills processes whose command line contains the game name: run such commands from a script file.
- Open from `tests/E2E-LAYERS-PLAN.md`: one S6 run saw a false "member left". If it repeats, treat it as a real finding.

## Final report

What was done, measured before and after timings for cross-OS, every new public name, anything skipped, and open questions.
