# Tests

## The e2e suite: BRP tests

The e2e suite lives in two workspace packages: `tests/lobby-room/`
(`lobby_room_e2e`) and `tests/starboard/` (`starboard_e2e`). Each package has an
`e2e` test target whose test files each start real game windows and drive them
through the Bevy Remote Protocol (BRP), so the test sees what a player sees.
The lobby package also holds the `freenet_harness` and `contract_wire` targets.

The old script-based scenarios (`starboard_*`, `lobby_room_*`, `friends_reconnect`,
`direct_dial`, `known_peers`, `freenet_relaunch`, the `e2e-ring*` and
`e2e-relaunch*` loops, and `ring-stats`) were deleted in F7a. `e2e-cross.yml` is
disabled until it is rebuilt on the BRP harness.

Networks, chosen with `E2E_NETWORK`:

- **fake** (default for `e2e-ui-no-network`): an in-process fake lobby. Fast, for
  reviewing the UI. Some tests need the real network and refuse fake by design;
  they count as skips.
- **mainnet** (`e2e-mainnet`): the public Freenet network. This is the real proof.
  Run the tests one at a time with a 90 s gap between them: back-to-back runs
  leave the previous test's peers still exiting the ring, and the next test then
  joins slowly or not at all.
- **direct-dial**: tests that dial known addresses on purpose. They ignore
  `E2E_NETWORK`.

## Run commands

```bash
just e2e-ui-no-network              # all e2e tests on fake, serially
just e2e-ui-no-network direct_dial  # tests whose name matches a pattern
just e2e-mainnet                    # all e2e tests on mainnet, 90 s apart
just e2e-mainnet direct_dial        # one test on mainnet
just e2e-loop direct_dial 5 90      # that mainnet run 5 times, 90 s between runs
```

The raw command behind `e2e-ui-no-network`:

```bash
DISPLAY=:99 VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json E2E_NETWORK=fake \
  cargo nextest run -p lobby_room_e2e -p starboard_e2e --test e2e -E 'test(//) and test(/^[^:]*::[^:]*$/)' --run-ignored all --all-features -j 1
```

Tip for a machine with no X server and no root: download Xvfb without installing
it (`apt-get download xvfb`, then `dpkg -x` into a scratch directory), start it with
`Xvfb :99 -screen 0 1280x800x24 -nolisten tcp`, and run the tests with `DISPLAY=:99`.

Direct cargo needs `CARGO_TARGET_DIR=$HOME/.cache/cargo-target/freenet-libp2p-bevy-plugin`
(jemalloc rejects the space in the default path). `just` sets it for you.

## Other test targets

- `freenet_harness`: CRDT-law checks of the lobby contract (`just freenet-harness`).
- `contract_wire`: wire-format checks of the contract.
- `tests/lobby-room/{github,local}_checkpoint_board.rs`: checkpoint boards with
  their own unit tests.
