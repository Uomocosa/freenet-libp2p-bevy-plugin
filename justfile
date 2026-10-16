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

lint:
    lele-lint .

bevy-lint:
    lele-bevy-lint .

contract:
    cargo build --manifest-path contract/lobby/Cargo.toml --release --target wasm32-unknown-unknown

# Contract CRDT-law suite (freenet-contract-harness) against the lobby contract.
freenet-harness:
    cargo nextest run --test freenet_harness --all-features --run-ignored all -- --nocapture

setup:
    prek install -t pre-commit -t pre-push

check:
    prek run --all-files

ci: build check
