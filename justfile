set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Build outside `[R] Rust` (jemalloc's configure rejects spaces); persistent, portable.
export CARGO_TARGET_DIR := env("HOME") / ".cache/cargo-target/freenet-libp2p-bevy-plugin"

# Toolchain: rust-toolchain.toml (nightly + wasm32-unknown-unknown).
# Extra tools: cargo-nextest, just, lele-lint / lele-bevy-lint.

# Headless Bevy previews use the software Vulkan driver for deterministic, clean teardown.
_lavapipe := `ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -n1`

# Log filter for e2e-mainnet: lobby_room=info keeps the tick markers the BRP
# checks read from the logs; freenet::config=info keeps the mainnet caption
# detection truthful.
e2e_log_filter := "warn,starboard=info,discovery=info,lobby_room=info,room_sync=info,freenet_libp2p_bevy_core::methods::embedded_node=info,freenet_libp2p_bevy_lobby_room::testing::bevy_systems=info,freenet::config=info,freenet::operations::connect=debug,freenet::ring=debug,freenet::operations::get=debug,freenet::operations::put=debug"

build:
    cargo build --workspace --all-targets --all-features

# every feature combination users can pick must compile
features:
    cargo check -p freenet-libp2p-bevy-core --no-default-features
    cargo check -p freenet-libp2p-bevy-core --no-default-features --features testing
    cargo check -p freenet-libp2p-bevy-core
    cargo check -p freenet-libp2p-bevy-lobby-room
    cargo check -p freenet-libp2p-bevy-lobby-room --features testing
    cargo check -p freenet-libp2p-bevy-plugin --no-default-features
    cargo check -p freenet-libp2p-bevy-plugin --no-default-features --features lobby-room
    cargo check -p freenet-libp2p-bevy-plugin --no-default-features --features embedded-node
    cargo check -p freenet-libp2p-bevy-plugin
    cargo check --workspace --all-targets --all-features

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
        current=$(find crates/lobby-room/bevy_plugin/ui_preview/_unsent -type f -name '*.png' -exec sha256sum {} + 2>/dev/null | sort -k2 || true)
        if [ "$run" -gt 1 ] && [ "$current" != "$previous" ]; then
            echo "UNSTABLE: run $run differs"
            diff <(echo "$previous") <(echo "$current") || true
            exit 1
        fi
        previous=$current
    done
    echo "STABLE across {{ N }} runs"

# UI layer on the fake network (BRP suite by default): fast, for reviewing the UI.
# Telegram stays silent. `just e2e-ui-no-network direct_dial` runs one name pattern.
e2e-ui-no-network TEST="brp_":
    #!/usr/bin/env bash
    set -euo pipefail
    # Headless-safe display defaults (overridable); the e2e peers need X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=fake TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= cargo nextest run -p e2e --no-fail-fast -j 1 --test e2e -E 'test(/{{ TEST }}/)' --all-features --run-ignored all -- --nocapture

# The matching BRP tests on the public network, one at a time with PAUSE seconds
# between them (the 90 s gap lets the previous test's peers leave the ring).
# Runs every matching test, then lists the failures and exits 1 if any. Telegram sends as usual.
e2e-mainnet TEST="brp_" PAUSE="90":
    #!/usr/bin/env bash
    set -euo pipefail
    # LOBBY_RUST_LOG default is loop-only; an exported value still wins.
    export LOBBY_RUST_LOG="${LOBBY_RUST_LOG:-{{ e2e_log_filter }}}"
    # Headless-safe display defaults (overridable); the e2e peers need X.
    export DISPLAY="${DISPLAY:-:0}"
    export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
    names="$(cargo nextest list -p e2e --test e2e --all-features --run-ignored all --message-format oneline -E 'test(/{{ TEST }}/)' 2>/dev/null | sed -n 's/^e2e::e2e //p')"
    if [ -z "$names" ]; then
        echo "e2e-mainnet: no test matches '{{ TEST }}'" >&2
        exit 1
    fi
    first=1
    failed=()
    while IFS= read -r name; do
        if [ "$first" = 0 ]; then sleep {{ PAUSE }}; fi
        first=0
        echo "e2e-mainnet: $name"
        if ! VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=mainnet cargo nextest run -p e2e --no-fail-fast -j 1 --test e2e -E "test(=$name)" --all-features --run-ignored all -- --nocapture; then
            failed+=("$name")
        fi
    done <<< "$names"
    if [ "${#failed[@]}" -gt 0 ]; then
        echo "e2e-mainnet: ${#failed[@]} failed:" >&2
        printf '  %s\n' "${failed[@]}" >&2
        exit 1
    fi
    echo "e2e-mainnet: all passed"

# Run `just e2e-mainnet TEST` RUNS times, PAUSE seconds between runs (never before the first).
e2e-loop TEST RUNS="5" PAUSE="90":
    #!/usr/bin/env bash
    set -euo pipefail
    bad=0
    for run in $(seq 1 {{ RUNS }}); do
        if [ "$run" -gt 1 ]; then sleep {{ PAUSE }}; fi
        echo "e2e-loop: run $run of {{ RUNS }}"
        if ! just e2e-mainnet "{{ TEST }}"; then bad=$((bad + 1)); fi
    done
    echo "e2e-loop: $bad of {{ RUNS }} runs had failures"
    [ "$bad" = 0 ]

# Run the starboard example detached (no terminal tie-up; only the app window).
# Extra args: --username, --token, --transport.
run-starboard *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p starboard
    mkdir -p .local-run
    log=.local-run/starboard.log
    setsid "$CARGO_TARGET_DIR/debug/starboard" {{ ARGS }} >"$log" 2>&1 &
    echo "starboard running detached — log: $log"

# Headless starboard benchmark (no window): 2 and 8 in-process peers, 60 strokes/s each,
# 20 s measured per scenario; overwrites examples/starboard/BENCH.md.
bench-starboard:
    cargo nextest run -p starboard --all-features --run-ignored all -E 'test(bench_starboard)' -- --nocapture

# Headless history measurement (no window): one room's Kademlia history size and
# put/get latency; overwrites examples/starboard/HISTORY.md.
bench-history:
    cargo nextest run -p starboard --all-features --run-ignored all -E 'test(history_bench)' -- --nocapture

# Run the lobby_room example detached (no terminal tie-up; only the app window).
# Extra args toggle ui/script/transport, e.g. `just run --ui custom --script "create:room-1, stay"`.
run *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p lobby_room
    mkdir -p .local-run
    log=.local-run/lobby-room.log
    setsid "$CARGO_TARGET_DIR/debug/lobby_room" {{ ARGS }} >"$log" 2>&1 &
    echo "lobby_room running detached — log: $log"

# Run the lobby_room example with the defaults (username lele, plugin default UI).
demo:
    just run --username lele --ui default

new-game-id:
    cargo run -p freenet-libp2p-bevy-core --bin new-game-id

lint:
    lele-lint crates/freenet-libp2p-bevy-core
    lele-lint crates/lobby-room/bevy_plugin
    lele-lint crates/freenet-libp2p-bevy-plugin

bevy-lint:
    lele-bevy-lint crates/freenet-libp2p-bevy-core
    lele-bevy-lint crates/lobby-room/bevy_plugin
    lele-bevy-lint crates/freenet-libp2p-bevy-plugin

# Regenerate the __methods__ indexes after adding or removing #[atomic_delegates] shells.
sync-methods:
    lele-lint --sync-methods crates/freenet-libp2p-bevy-core
    lele-lint --sync-methods crates/lobby-room/bevy_plugin
    lele-lint --sync-methods crates/lobby-room/freenet_contract
    lele-lint --sync-methods crates/lobby-room/shared_types

# The only way to rebuild the lobby contract wasm. build.rs was removed because it
# inherited RUSTC_WORKSPACE_WRAPPER=clippy-driver from `cargo clippy`, which rebuilt
# the same source to different bytes. This recipe clears the wrappers so the output
# is repeatable, then copies the result over the canonical committed file.
contract:
    #!/usr/bin/env bash
    set -euo pipefail
    env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u CLIPPY_ARGS cargo build -p lobby_contract --release --target wasm32-unknown-unknown
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/lobby_contract.wasm" crates/lobby-room/freenet_contract/lobby_contract.wasm
    wc -c crates/lobby-room/freenet_contract/lobby_contract.wasm
    sha256sum crates/lobby-room/freenet_contract/lobby_contract.wasm

# The lobby contract is a workspace member: `just fmt` (`cargo fmt --all`) and
# `just clippy` (`--workspace --all-targets --all-features`) already cover it.
# Only the per-crate lele-lint stays here.
contract-lint:
    lele-lint crates/lobby-room/freenet_contract
    lele-lint crates/lobby-room/shared_types

# common_types is a workspace member too: `just fmt`, `just clippy` and
# `just test` (`--workspace --all-targets --all-features`) already cover it.
# Only the per-crate lele-lint and the wasm check stay here.
common-types-wasm:
    cargo check -p common_types --target wasm32-unknown-unknown
    cargo check -p lobby_room_shared_types --target wasm32-unknown-unknown

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
