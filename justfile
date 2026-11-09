set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Build outside `[R] Rust` (jemalloc's configure rejects spaces); persistent, portable.
export CARGO_TARGET_DIR := env("HOME") / ".cache/cargo-target/freenet-libp2p-bevy-plugin"

# Toolchain: rust-toolchain.toml (nightly + wasm32-unknown-unknown).
# Extra tools: cargo-nextest, just, lele-lint / lele-bevy-lint.

# Headless Bevy previews use the software Vulkan driver for deterministic, clean teardown.
_lavapipe := `ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -n1`

# Shared e2e log filter: lobby_room=info keeps the script/tick markers the
# scenario checks read; freenet::config=info keeps the mainnet/local caption
# detection truthful.
e2e_log_filter := "warn,script=info,starboard=info,discovery=info,lobby_room=info,freenet::config=info,freenet::operations::connect=debug,freenet::ring=debug,freenet::operations::get=debug,freenet::operations::put=debug"

build:
    cargo build --workspace --all-targets --all-features

# every feature combination users can pick must compile
features:
    cargo check -p freenet-libp2p-bevy-plugin --no-default-features
    cargo check -p freenet-libp2p-bevy-plugin --no-default-features --features discovery
    cargo check -p freenet-libp2p-bevy-plugin
    cargo check -p freenet-libp2p-bevy-plugin --all-targets --all-features

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
    cargo fmt --all -- --check

test:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --workspace --all-targets --all-features

# Re-send every UI preview to Telegram, ignoring the change gate.
resend-all-previews:
    LELE_PREVIEW_RESEND_ALL=1 VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --workspace --all-targets --all-features -E 'test(/_preview$/)'

# Render every preview N times without sending; fail if any render differs between runs.
preview-stability N="5":
    #!/usr/bin/env bash
    set -euo pipefail
    previous=""
    for run in $(seq 1 {{ N }}); do
        TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= LELE_PREVIEW_DRY_RUN=1 VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --workspace --all-targets --all-features -E 'test(/_preview$/)'
        current=$(find crates/freenet-libp2p-bevy-plugin/ui_preview/_unsent -type f -name '*.png' -exec sha256sum {} + 2>/dev/null | sort -k2 || true)
        if [ "$run" -gt 1 ] && [ "$current" != "$previous" ]; then
            echo "UNSTABLE: run $run differs"
            diff <(echo "$previous") <(echo "$current") || true
            exit 1
        fi
        previous=$current
    done
    echo "STABLE across {{ N }} runs"

# Real, visual e2e: needs X + the freenet network + N lobby_room windows.
e2e-discovery N="3":
    VK_ICD_FILENAMES={{ _lavapipe }} E2E_PEERS={{ N }} cargo nextest run -p e2e --no-fail-fast --test lobby_room_discovery -E 'test(lobby_room_discovery)' --all-features --run-ignored all -- --nocapture

# Ring health only: no room, no UI, no libp2p mesh — just N embedded Freenet nodes.
e2e-ring N:
    VK_ICD_FILENAMES={{ _lavapipe }} E2E_PEERS={{ N }} cargo nextest run -p e2e --no-fail-fast --test lobby_room_discovery -E 'test(freenet_ring)' --all-features --run-ignored all -- --nocapture

# The whole ladder, one network at a time (never 4 x N nodes on one NAT at once).
e2e-ring-ladder:
    #!/usr/bin/env bash
    set -euo pipefail
    for peers in 2 3 4 5; do just e2e-ring "$peers"; done

# Per-peer ring timing stats over .local-run/<scenario>-<epoch> runs.
# Examples: `just ring-stats`, `just ring-stats --last 8`,
# `just ring-stats --scenario freenet_ring --last 3 --json /tmp/ring.json`,
# `just ring-stats --since 1793900000` (only runs from that epoch on).
ring-stats *ARGS:
    python3 scripts/ring_stats.py {{ ARGS }}

# Repeat `just e2e-ring N` up to RUNS times; stop at the first failure and
# always print the stats for the runs completed so far.
# KEEP_GOING=1 is for measuring a baseline only: it runs all RUNS even when
# some fail, then exits non-zero with the failure count.
#
# Pause PAUSE seconds (or E2E_LOOP_PAUSE_SECS) BETWEEN runs: never before the
# first, never after the last. The pause lets the previous run's peers leave
# the mainnet ring before the next run joins: with a 90 s pause 8 of 8 runs
# passed (5 of 24 peers over 30 s to join); back-to-back runs gave 7 of 8
# (11 of 24 peers slow or never joined), because the previous run's peers had
# just exited.
#
# One Telegram summary per batch (stats plus the clip of each failed run);
# per-run Telegram stays silent as today. The silencing (empty creds) is an
# inline prefix on the per-run call only, so the summary still sees the real
# credentials. Sending is best-effort: a Telegram failure only prints a
# warning. E2E_LOOP_NOTIFY=0 skips the send (used by e2e-ring-check, which
# sends its own combined summary for both halves).
e2e-ring-loop N="3" RUNS="5" KEEP_GOING="0" PAUSE="90":
    #!/usr/bin/env bash
    set -euo pipefail
    # Telegram stays silent per run: empty creds make send_video_best_effort a no-op
    # (see the preview-stability recipe and telegram-bot-helper from_env).
    # LOBBY_RUST_LOG default is loop-only; an exported value still wins.
    export LOBBY_RUST_LOG="${LOBBY_RUST_LOG:-{{ e2e_log_filter }}}"
    # Headless-safe display defaults (overridable); the e2e peers need X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    pause="${E2E_LOOP_PAUSE_SECS:-{{ PAUSE }}}"
    # Only this loop's runs are reported: run dirs carry a unix-epoch suffix
    # (see epoch_secs in tests/e2e/tests/common/scenario.rs), so --since filters out
    # older runs instead of showing stale data.
    started=$(date +%s)
    failed=0
    run=0
    while [ "$run" -lt {{ RUNS }} ]; do
        run=$((run + 1))
        if [ "$run" -gt 1 ]; then sleep "$pause"; fi
        if TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= just e2e-ring {{ N }}; then
            :
        else
            failed=$((failed + 1))
            echo "e2e-ring-loop: run $run of {{ RUNS }} FAILED (N={{ N }})" >&2
            if [ "{{ KEEP_GOING }}" = "0" ]; then
                break
            fi
        fi
    done
    just ring-stats --since "$started"
    E2E_SUMMARY="$(python3 scripts/ring_stats.py --scenario freenet_ring --since "$started" --summary --batch "e2e-ring-loop N={{ N }}")" just _e2e-notify-text
    just _e2e-notify-clips freenet_ring "$started"
    if [ "$failed" -ne 0 ]; then
        if [ "{{ KEEP_GOING }}" = "1" ]; then
            echo "e2e-ring-loop: $failed of {{ RUNS }} failed (N={{ N }})" >&2
        else
            echo "e2e-ring-loop: run $run of {{ RUNS }} FAILED (N={{ N }})" >&2
        fi
        exit 1
    fi
    echo "e2e-ring-loop: all {{ RUNS }} runs passed (N={{ N }})"

# N=1 then N=3, 5 runs each, each stopping at the first failure.
# Inherits PAUSE (or E2E_LOOP_PAUSE_SECS), including one pause between the N=1
# and N=3 halves. Sends a single combined Telegram summary for both halves;
# the halves themselves stay silent (E2E_LOOP_NOTIFY=0).
# Stop after N=1 if it fails (the summary still goes out).
e2e-ring-check PAUSE="90":
    #!/usr/bin/env bash
    set -euo pipefail
    pause="${E2E_LOOP_PAUSE_SECS:-{{ PAUSE }}}"
    started=$(date +%s)
    failed=0
    if ! E2E_LOOP_NOTIFY=0 just e2e-ring-loop 1 5 0 "$pause"; then
        failed=1
    fi
    if [ "$failed" -eq 0 ]; then
        sleep "$pause"
        if ! E2E_LOOP_NOTIFY=0 just e2e-ring-loop 3 5 0 "$pause"; then
            failed=1
        fi
    fi
    just ring-stats --since "$started"
    E2E_SUMMARY="$(python3 scripts/ring_stats.py --scenario freenet_ring --since "$started" --summary --batch "e2e-ring-check N=1,3")" just _e2e-notify-text
    just _e2e-notify-clips freenet_ring "$started"
    if [ "$failed" -ne 0 ]; then
        echo "e2e-ring-check: a half FAILED" >&2
        exit 1
    fi
    echo "e2e-ring-check: both halves passed"

# Saved-node relaunch: one peer launches cold, exits, waits GAP seconds with
# the app closed, then launches again with the same node folder and game
# token (see tests/e2e/tests/freenet_relaunch.rs and tests/README.md).
e2e-relaunch GAP="10":
    #!/usr/bin/env bash
    set -euo pipefail
    # Telegram stays silent: empty creds make send_video_best_effort a no-op.
    # LOBBY_RUST_LOG default is loop-only; an exported value still wins.
    export LOBBY_RUST_LOG="${LOBBY_RUST_LOG:-{{ e2e_log_filter }}}"
    # Headless-safe display defaults (overridable); the e2e peer needs X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    VK_ICD_FILENAMES={{ _lavapipe }} E2E_RELAUNCH_GAP_SECS={{ GAP }} TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= cargo nextest run -p e2e --no-fail-fast --test freenet_relaunch -E 'test(freenet_relaunch)' --all-features --run-ignored all -- --nocapture

# Repeat `just e2e-relaunch GAP` up to RUNS pairs; stop at the first failure
# and always print the stats for the cold and warm runs completed so far.
#
# Same contract as e2e-ring-loop: PAUSE seconds (or E2E_LOOP_PAUSE_SECS)
# BETWEEN pairs (never before the first, never after the last; see the
# numbers quoted there) and one Telegram summary per batch, best-effort.
e2e-relaunch-loop GAP="10" RUNS="5" PAUSE="90":
    #!/usr/bin/env bash
    set -euo pipefail
    # LOBBY_RUST_LOG default is loop-only; an exported value still wins.
    export LOBBY_RUST_LOG="${LOBBY_RUST_LOG:-{{ e2e_log_filter }}}"
    # Headless-safe display defaults (overridable); the e2e peer needs X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    pause="${E2E_LOOP_PAUSE_SECS:-{{ PAUSE }}}"
    # Only this loop's runs are reported: run dirs carry a unix-epoch suffix
    # (see epoch_secs in tests/e2e/tests/common/scenario.rs), so --since filters out
    # older runs instead of showing stale data.
    started=$(date +%s)
    failed=0
    run=0
    while [ "$run" -lt {{ RUNS }} ]; do
        run=$((run + 1))
        if [ "$run" -gt 1 ]; then sleep "$pause"; fi
        if TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= just e2e-relaunch {{ GAP }}; then
            :
        else
            failed=$((failed + 1))
            echo "e2e-relaunch-loop: run $run of {{ RUNS }} FAILED (GAP={{ GAP }})" >&2
            break
        fi
    done
    just ring-stats --scenario freenet_relaunch_cold --since "$started"
    just ring-stats --scenario freenet_relaunch_warm --since "$started"
    cold="$(python3 scripts/ring_stats.py --scenario freenet_relaunch_cold --since "$started" --summary --batch "e2e-relaunch-loop GAP={{ GAP }} pairs $((run - failed)) of {{ RUNS }} cold")"
    warm="$(python3 scripts/ring_stats.py --scenario freenet_relaunch_warm --since "$started" --summary --batch "e2e-relaunch-loop GAP={{ GAP }} pairs $((run - failed)) of {{ RUNS }} warm")"
    E2E_SUMMARY="$(printf '%s\n%s' "$cold" "$warm")" just _e2e-notify-text
    just _e2e-notify-clips freenet_relaunch_cold "$started"
    just _e2e-notify-clips freenet_relaunch_warm "$started"
    if [ "$failed" -ne 0 ]; then
        echo "e2e-relaunch-loop: run $run of {{ RUNS }} FAILED (GAP={{ GAP }})" >&2
        exit 1
    fi
    echo "e2e-relaunch-loop: all {{ RUNS }} runs passed (GAP={{ GAP }})"

# Internal: send one batch summary via Telegram. The text arrives in E2E_SUMMARY
# (an argument would re-enter just interpolation); the send is best-effort and
# never fails the caller. E2E_LOOP_NOTIFY=0 prints but does not send.
_e2e-notify-text:
    #!/usr/bin/env bash
    set -euo pipefail
    printf '%s\n' "$E2E_SUMMARY"
    if [ "${E2E_LOOP_NOTIFY:-1}" = "0" ]; then
        echo "(telegram send skipped: E2E_LOOP_NOTIFY=0)" >&2
        exit 0
    fi
    if ! cargo run -q -p freenet-libp2p-bevy-plugin --example telegram_send -- --text "$E2E_SUMMARY"; then
        echo "warning: telegram summary failed" >&2
    fi

# Internal: send the clip.mp4 of each failed run in SCENARIO since SINCE via
# Telegram, captioned with the run name and failure reason. Best-effort and
# never fails the caller. E2E_LOOP_NOTIFY=0 skips the send.
_e2e-notify-clips SCENARIO SINCE:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "${E2E_LOOP_NOTIFY:-1}" = "0" ]; then
        echo "(telegram clips skipped: E2E_LOOP_NOTIFY=0)" >&2
        exit 0
    fi
    clips="$(python3 scripts/ring_stats.py --scenario "{{ SCENARIO }}" --since "{{ SINCE }}" --failed-clips || true)"
    while IFS="$(printf '\t')" read -r clip caption; do
        [ -n "$clip" ] || continue
        if ! cargo run -q -p freenet-libp2p-bevy-plugin --example telegram_send -- --video "$clip" --caption "$caption"; then
            echo "warning: telegram clip send failed ($clip)" >&2
        fi
    done <<<"$clips"

e2e-join:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run -p e2e --no-fail-fast --test lobby_room_join --all-features --run-ignored all -- --nocapture

e2e-room:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run -p e2e --no-fail-fast --test room_messaging --all-features --run-ignored all -- --nocapture

# Real, visual e2e: needs X + the freenet network + 3 starboard windows.
e2e-starboard:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run -p e2e --no-fail-fast --test starboard_e2e --all-features --run-ignored all -- --nocapture

# Real, visual e2e: the Kademlia history scenarios (S4a, S4b, S4c) on one machine.
e2e-history:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run -p e2e --no-fail-fast --test starboard_history --all-features --run-ignored all -- --nocapture

# Real, visual e2e: S3, S5, S6, S7 on one machine.
e2e-scenarios:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run -p e2e --no-fail-fast --test starboard_scenarios --all-features --run-ignored all -- --nocapture

# UI layer on the fake network: fast, for reviewing the UI; Telegram stays silent.
e2e-ui-no-network TEST="":
    #!/usr/bin/env bash
    set -euo pipefail
    # Headless-safe display defaults (overridable); the e2e peers need X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    if [ -n "{{ TEST }}" ]; then
        VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=fake TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= cargo nextest run -p e2e --no-fail-fast --test e2e -E 'test(/{{ TEST }}/)' --all-features --run-ignored all -- --nocapture
    else
        VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=fake TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= cargo nextest run -p e2e --no-fail-fast --test e2e --all-features --run-ignored all -- --nocapture
    fi

# The same scenarios on the public network: the real proof; Telegram sends as usual.
e2e-mainnet TEST="":
    #!/usr/bin/env bash
    set -euo pipefail
    # LOBBY_RUST_LOG default is loop-only; an exported value still wins.
    export LOBBY_RUST_LOG="${LOBBY_RUST_LOG:-{{ e2e_log_filter }}}"
    # Headless-safe display defaults (overridable); the e2e peers need X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    if [ -n "{{ TEST }}" ]; then
        VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=mainnet cargo nextest run -p e2e --no-fail-fast --test e2e -E 'test(/{{ TEST }}/)' --all-features --run-ignored all -- --nocapture
    else
        VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=mainnet cargo nextest run -p e2e --no-fail-fast --test e2e --all-features --run-ignored all -- --nocapture
    fi

# Run the starboard example detached (no terminal tie-up; only the app window).
# Extra args: --username, --token, --transport.
run-starboard *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p freenet-libp2p-bevy-plugin --features e2e --example starboard
    mkdir -p .local-run
    log=.local-run/starboard.log
    setsid "$CARGO_TARGET_DIR/debug/examples/starboard" {{ ARGS }} >"$log" 2>&1 &
    echo "starboard running detached — log: $log"

# Headless starboard benchmark (no window): 2 and 8 in-process peers, 60 strokes/s each,
# 20 s measured per scenario; overwrites examples/starboard/BENCH.md.
bench-starboard:
    cargo nextest run -p freenet-libp2p-bevy-plugin --example starboard --all-features --run-ignored all -E 'test(bench_starboard)' -- --nocapture

# Headless history measurement (no window): one room's Kademlia history size and
# put/get latency; overwrites examples/starboard/HISTORY.md.
bench-history:
    cargo nextest run -p freenet-libp2p-bevy-plugin --example starboard --all-features --run-ignored all -E 'test(history_bench)' -- --nocapture

# Run the lobby_room example detached (no terminal tie-up; only the app window).
# Extra args toggle ui/script/transport, e.g. `just run --ui custom --script "create:room-1, stay"`.
run *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p freenet-libp2p-bevy-plugin --features e2e --example lobby_room
    mkdir -p .local-run
    log=.local-run/lobby-room.log
    setsid "$CARGO_TARGET_DIR/debug/examples/lobby_room" {{ ARGS }} >"$log" 2>&1 &
    echo "lobby_room running detached — log: $log"

# Run the lobby_room example with the defaults (username lele, plugin default UI).
demo:
    just run --username lele --ui default

new-game-id:
    cargo run -p freenet-libp2p-bevy-plugin --bin new-game-id

lint:
    lele-lint crates/freenet-libp2p-bevy-plugin

bevy-lint:
    lele-bevy-lint crates/freenet-libp2p-bevy-plugin

# Regenerate the __methods__/src/mod.rs + __methods__/ui/mod.rs indexes after adding or removing #[atomic_delegates] shells.
sync-methods:
    lele-lint --sync-methods crates/freenet-libp2p-bevy-plugin

# The only way to rebuild the lobby contract wasm. build.rs was removed because it
# inherited RUSTC_WORKSPACE_WRAPPER=clippy-driver from `cargo clippy`, which rebuilt
# the same source to different bytes. This recipe clears the wrappers so the output
# is repeatable, then copies the result over the canonical committed file.
contract:
    #!/usr/bin/env bash
    set -euo pipefail
    env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u CLIPPY_ARGS cargo build -p lobby_contract --release --target wasm32-unknown-unknown
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/lobby_contract.wasm" crates/lobby_contract/lobby_contract.wasm
    wc -c crates/lobby_contract/lobby_contract.wasm
    sha256sum crates/lobby_contract/lobby_contract.wasm

# The lobby contract is a workspace member, but its checks still run here too,
# from the repo root so rustup picks up rust-toolchain.toml.
contract-fmt:
    cargo fmt -p lobby_contract -- --check

contract-clippy:
    cargo clippy -p lobby_contract --all-targets --all-features -- -D warnings

contract-lint:
    lele-lint crates/lobby_contract

# common_types is a workspace member too. Same shape as the contract recipes above.
common-types-fmt:
    cargo fmt -p common_types -- --check

common-types-clippy:
    cargo clippy -p common_types --all-targets --all-features -- -D warnings

common-types-test:
    cargo nextest run -p common_types --all-features

common-types-wasm:
    cargo check -p common_types --target wasm32-unknown-unknown

common-types-lint:
    lele-lint crates/common_types

# Contract CRDT-law suite (freenet-contract-harness) against the lobby contract.
freenet-harness:
    cargo nextest run -p e2e --test freenet_harness --all-features --run-ignored all -- --nocapture

setup:
    prek install -t pre-commit -t pre-push

check:
    prek run --all-files

ci: build features check
