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

- **fake** (default for `e2e`): an in-process fake lobby. Fast, for
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
just e2e                    # all e2e tests on fake, serially
just e2e direct_dial        # tests whose name matches a pattern
just e2e-mainnet            # all e2e tests on mainnet, 90 s apart
just e2e-mainnet direct_dial  # one test on mainnet
just e2e-mainnet direct_dial 90 5  # that mainnet run 5 times, 90 s between runs
```

The raw command behind `e2e`:

```bash
VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json E2E_NETWORK=fake \
  cargo nextest run -p lobby_room_e2e -p starboard_e2e --test e2e -E 'test(//) and test(/^[^:]*::[^:]*$/)' --run-ignored all --all-features -j 1
```

## The display

On Linux the harness starts a private `Xvfb` for every run (`:90`..`:120`,
`1442x800x24`, `-nolisten tcp`) and gives the peer processes its `DISPLAY`; it kills
the server when the run ends. Your own `DISPLAY` is never used.

- `E2E_XVFB=/path/to/Xvfb` picks the binary; otherwise `Xvfb` must be on `PATH`.
- `E2E_DISPLAY=:99` reuses a server that is already running instead of starting one.

Tip for a machine with no X server and no root: download Xvfb without installing it
(`apt-get download xvfb`, then `dpkg -x` into a scratch directory) and point
`E2E_XVFB` at the extracted binary.

## Recording

Every e2e test can record its screen and post the video to Telegram:

```bash
just e2e-record              # fake network, records and sends every test
just e2e-record two_peers    # one test
E2E_RECORD_DRY_RUN=1 just e2e-record   # encode, keep the file, do not send
just e2e-mainnet                # records by default (E2E_RECORD=0 turns it off)
```

A test opts in with two lines: `let mut recording = net.recording("<test name>");`
after the network is prepared, and `recording.mark_passed();` as the last statement
before `Ok(())`.

- One clip per test at `.local-run/brp-<stamp>/e2e-<test name>.mp4` (1442x800, 10
  fps, H.264). `.local-run/` is gitignored.
- The caption is `<test name> PASSED|FAILED — <duration>`. The verdict starts as
  `FAILED`; only `mark_passed()` turns it green, so an early `?`, a `return Err` or a
  panic produces a `FAILED` caption.
- `E2E_RECORD=1` starts the recorder; without it `net.recording` returns an inert
  guard and no ffmpeg process is spawned.
- `E2E_RECORD_DRY_RUN=1` prints the clip path and caption instead of sending.
  Telegram is silent when `TELEGRAM_BOT_TOKEN`/`TELEGRAM_CHAT_ID` are empty.
- `ffmpeg` (with libx264) comes from `E2E_FFMPEG` or `PATH`. A clip above 45 MiB is
  re-encoded once at a higher CRF; if it is still too large it is skipped with a
  message. Recording is Linux-only for now: Windows will capture the desktop with
  gdigrab later.

Direct cargo needs `CARGO_TARGET_DIR=$HOME/.cache/cargo-target/freenet-libp2p-bevy-plugin`
(jemalloc rejects the space in the default path). `just` sets it for you.

Use `just --list`; raw cargo needs CARGO_TARGET_DIR (space in path), so prefer the recipes (`just t`, `just check-crate`).

## Other test targets

- `freenet_harness`: CRDT-law checks of the lobby contract (`just freenet-harness`).
- `contract_wire`: wire-format checks of the contract.
- `tests/lobby-room/{github,local}_checkpoint_board.rs`: checkpoint boards with
  their own unit tests.
