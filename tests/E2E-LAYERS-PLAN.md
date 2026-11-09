# E2E layers plan — one test file, four networks, cross-OS

## 0. Goal

Each e2e test is one file that says what the peers do. When it runs, we pick the network:

```text
just e2e-ui-no-network <test>    fake network, fast, for reviewing the UI
just e2e-local-gateways <test>   private Freenet network on localhost
just e2e-mainnet <test>          the public network: the real proof
```

A fourth network covers peers that already know each other:

```text
#[e2e_test(game = "starboard", network = "direct-dial")]   real libp2p, no Freenet node
```

The pin beats `E2E_NETWORK`. The direct-dial tests (`direct_dial`,
`friends_reconnect`, `known_peers`) dial known addresses and prove dialing,
friending and reconnecting work. Mainnet stays the only proof of connecting
with no prior knowledge.

The same test also runs across Linux and Windows from a GitHub workflow.

## 1. Decisions (agreed)

| # | Topic | Decision |
|---|---|---|
| L1 | Peers | One process per peer. Each example stays an app that can run alone; tests launch it. |
| L2 | Fake layer | `testing::FakeNetworkPlugin` replaces only the libp2p swarm and the Freenet node. Scripts, UI, the discovery state machine and `room_sync` stay real. |
| L3 | Orchestrator | `testing::Orchestrator` starts peers, watches them, routes fake traffic and collects results. Its routing lives in a private `PeerMessageRouter` with no I/O. Peers reach it over a localhost socket. |
| L4 | Steps | `E2EStep` (rename of `SharedScriptStep`), `ScriptStep` (parsing and text) and `PollableScriptStep` (running) live in `src/testing/`. |
| L5 | Game steps | The definition (`StarboardStep`, step groups) lives in `tests/e2e_steps/`; the example depends on it as a crate. How a step runs stays in the example. Step types use no game types. |
| L6 | Tests | `tests/e2e/main.rs` is one test target, with one file per scenario, using `#[e2e_test]`. Rooms and deadlines come from `Orchestrator`. |
| L7 | Sync between peers | Named checkpoints: `mark_completed(PAINTING_DONE)` and `.starts_after(PAINTING_DONE)`. No step numbers. |
| L8 | Examples | The real plugin setup stays visible in `main()`. Testing adds `testing::NetworkArgs`, `testing::E2EArgs<Step>` and one `FakeNetworkPlugin::from_args` line before the plugins. The user may refactor this later. |
| L9 | Seams | `p2p::SwarmTaskOverride` and `discovery::LobbyTaskOverride`, compiled only with the `testing` feature. |
| L10 | Feature | `src/testing/` is behind a `testing` feature, enabled by `e2e`. It adds no heavy dependency: `freenet` stays a dev-dependency and `start_embedded_freenet_node` stays in `examples/shared/`. |
| L11 | Default network | Tests default to no-network, so a plain test run can never reach mainnet by accident. An app run alone defaults to `--network mainnet`, because the fake needs an `Orchestrator`. |
| L12 | Cross-OS | Every machine runs the same test with its own `Orchestrator` and only its own peers (`E2E_MACHINE`, `E2E_PLACEMENT`). Discovery tests share only checkpoint names through a `CheckpointBoard`; direct-dial tests also share peer addresses through it. |
| L13 | Board | `CheckpointBoard` trait and `LocalCheckpointBoard` in `src/testing/`; `GitHubCheckpointBoard` in `tests/e2e/`, probably on commit statuses (untested). |
| L14 | Verdict | The `check` job runs the same test with `E2E_ROLE=check`: it starts no peers and evaluates the test's expectations over the downloaded results. |
| L15 | Bundle | The compiled `tests/e2e` program joins the bundle, so later runs need no cargo. One worker on the Windows PC. |

The HARD RULE in `tests/README.md` applies to `e2e-mainnet` and `e2e-local-gateways`: peers meet only through the lobby contract. In the fake layer, the same rule holds through the fake lobby. Direct-dial tests are exempt: they dial known addresses on purpose.

## 2. Test method names

`create_room()`, `join_room()`, `wait_until_room_has_members(n)`, `paint(STROKES)`,
`erase(STROKES)`, `mark_completed(CHECKPOINT)`, `wait_until_canvas_has_content()`,
`wait_until_canvas_stops_changing()`, `exit_app()`, `.starts_after(CHECKPOINT)`,
`.within(duration)`, `test.expect_canvases_converge()`.

`wait_until_canvas_stops_changing` proves nothing about peers matching: an empty canvas also
stops changing, so a joiner waits for content first.

## 3. Evidence from the spike (2026-11-07)

The spike code is in `.local-run/spike/` (ignored by git).

- The real `DiscoveryState` and real `room_sync` converged over in-memory routing for create, join, paint and erase, with no `src/` changes. The test called the state machine directly: no `Config::run`, timers, scripts or UI.
- Not yet covered by routing: leaving, history, replica slices, cursors, late joins, crashes.
- Commands the spike's hub used: `Dial`, `AddKadPeer`, `PeerExchange`, `Subscribe`, `Publish`, `FetchSnapshot`, `RespondSnapshot`, `MeasureHistoryStore`.
- In-process peers would lose real pointer clicks, which is why L1 uses processes.

## 4. Order of work

Each phase leaves `just clippy`, `just lint`, `just bevy-lint` and `just test` green.

- [x] 1. `src/testing/` with the `testing` feature; move the script system there; add `ScriptStep`, `PollableScriptStep`, `E2EStep`, one generic `run_script`. Behaviour unchanged.
- [x] 2. `PeerMessageRouter` (no I/O), starting from `.local-run/spike/spike_q3.rs`, with unit tests for every `Libp2pCommand`.
- [x] 3. Override seams, `Orchestrator` socket, `FakeNetworkPlugin`, `NetworkArgs`, `E2EArgs`; both examples switch to them.
- [ ] 4. `tests/e2e_steps/`, `tests/e2e/tests/`, `#[e2e_test]`, checkpoints; port S1 first, then the rest. Done: S1, late join, S3-S7, all passed on fake and mainnet (2026-11-07). Left: `lobby_room` tests.
- [x] 5. Recipes `e2e-ui-no-network` and `e2e-mainnet`; S1 and late join passed on mainnet (2026-11-07).
- [ ] 6. `e2e-local-gateways`.
- [ ] 7. Placement, `CheckpointBoard`, `E2E_ROLE=check`, test program in the bundle, `e2e-cross.yml` update.
- [ ] 8. Remove the old `tests/e2e/tests/*.rs` scenario targets and `tests/e2e/tests/common/scenario.rs` once every scenario is ported.

## 5. Risks

- Open: one fake run of S6 saw a false "member left" before the crash, so the crasher missed the painted strokes. It did not recur (3/3 fake, 1/1 mainnet) and the cause is unproven: treat a repeat as a real finding, not a flake.
- The fake layer cannot find real connection bugs (the S5 duplicate-connection bug lived in libp2p) or NAT and Freenet timing. Mainnet stays the proof.
- Phases 1 and 3 change code the mainnet tests use, so S1 must pass on mainnet afterwards.
- `lele-bevy-preview` is not used for this layer: runs are recorded with ffmpeg as today.
