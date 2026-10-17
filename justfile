set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Build outside `[R] Rust` (jemalloc's configure rejects spaces); persistent, portable.
export CARGO_TARGET_DIR := env("HOME") / ".cache/cargo-target/freenet-libp2p-bevy-plugin"

# Toolchain: rust-toolchain.toml (nightly + wasm32-unknown-unknown).
# Extra tools: cargo-nextest, just, lele-lint / lele-bevy-lint.

# Headless Bevy previews use the software Vulkan driver for deterministic, clean teardown.
_lavapipe := `ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -n1`

build:
    cargo build --all-targets --all-features

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo fmt -- --check

test:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --all-targets --all-features

# Re-send every UI preview to Telegram, ignoring the change gate.
resend-all-previews:
    LELE_PREVIEW_RESEND_ALL=1 VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --all-targets --all-features -E 'test(/_preview$/)'

# Real, visual e2e: needs X + the freenet network + N lobby_room windows.
e2e-discovery:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --test lobby_room_discovery --all-features --run-ignored all -- --nocapture

e2e-join:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --test lobby_room_join --all-features --run-ignored all -- --nocapture

e2e-room:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --test room_messaging --all-features --run-ignored all -- --nocapture

# Run the lobby_room example detached (no terminal tie-up; only the app window).
# Extra args toggle ui/action/transport, e.g. `just run --ui custom --action create:room-1`.
run *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build --features e2e --example lobby_room
    mkdir -p .local-run
    log=.local-run/lobby-room.log
    setsid "$CARGO_TARGET_DIR/debug/examples/lobby_room" {{ ARGS }} >"$log" 2>&1 &
    echo "lobby_room running detached — log: $log"

# Run the lobby_room example with the defaults (username lele, plugin default UI).
demo:
    just run --username lele --ui default

lint:
    lele-lint .

bevy-lint:
    lele-bevy-lint .

contract:
    cargo build --manifest-path contract/lobby/Cargo.toml --release --target wasm32-unknown-unknown

# The lobby contract is its own workspace root (`[workspace]` in its Cargo.toml), so the
# parent's fmt/clippy/lint never reach it. Run each check here too, from the repo root so
# rustup picks up rust-toolchain.toml.
contract-fmt:
    cargo fmt --manifest-path contract/lobby/Cargo.toml -- --check

contract-clippy:
    cargo clippy --manifest-path contract/lobby/Cargo.toml --all-targets --all-features -- -D warnings

contract-lint:
    lele-lint --config lele.toml contract/lobby

# Contract CRDT-law suite (freenet-contract-harness) against the lobby contract.
freenet-harness:
    cargo nextest run --test freenet_harness --all-features --run-ignored all -- --nocapture

setup:
    prek install -t pre-commit -t pre-push

check:
    prek run --all-files

ci: build check
