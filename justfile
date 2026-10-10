set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

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

lint:
    lele-lint .

bevy-lint:
    lele-bevy-lint .

contract:
    cargo build --manifest-path contract/lobby/Cargo.toml --release --target wasm32-unknown-unknown

harness:
    cargo nextest run --test mainnet_local --all-features --run-ignored all -- --nocapture

hooks:
    git config core.hooksPath .githooks
    chmod +x .githooks/*

ci: build fmt clippy test
