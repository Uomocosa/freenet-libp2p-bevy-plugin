[unix]
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

[windows]
set shell := ["powershell.exe", "-NoLogo", "-Command"]

# Build outside `[R] Rust` (jemalloc's configure rejects spaces); persistent, portable.
export CARGO_TARGET_DIR := env("HOME") / ".cache/cargo-target/freenet-libp2p-bevy-plugin"

# Toolchain: rust-toolchain.toml (nightly + wasm32-unknown-unknown).
# Extra tools: cargo-nextest, just, lele-lint / lele-bevy-lint.

# Headless Bevy previews use the software Vulkan driver for deterministic, clean teardown.
# Linux-only path: just evaluates only the taken branch's backtick, so non-Linux
# shells never run the `ls` (verified on 1.58: untaken `exit 1` backtick is skipped).
_lavapipe := if os() == "linux" { `ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -n1` } else { "" }

# Log filter for e2e-mainnet: lobby_room=info keeps the tick markers the BRP
# checks read from the logs; freenet::config=info keeps the mainnet caption
# detection truthful.
e2e_log_filter := "warn,starboard=info,discovery=info,lobby_room=info,replica_sync=info,freenet_libp2p_bevy_core::methods::embedded_node=info,freenet_libp2p_bevy_lobby_room::testing::bevy_systems=info,freenet::config=info,freenet::operations::connect=debug,freenet::ring=debug,freenet::operations::get=debug,freenet::operations::put=debug"

[doc('Show this recipe list.')]
default:
    @just --list --unsorted

[group('dev')]
[doc('Build the whole workspace (all targets, all features).')]
build:
    cargo build --workspace --all-targets --all-features

# PATTERN filters on the full test path; empty matches everything.
[group('dev')]
[doc('Run workspace tests matching PATTERN (nextest; empty matches all).')]
t PATTERN="" *FLAGS:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --workspace --all-targets --all-features -E 'test(/{{ PATTERN }}/)' {{ FLAGS }}

# The worker profile hides the `*_plugin::tests` suites, which assemble whole apps, and the
# `*_preview` encode tests (the orchestrator's full `just test` covers them); the rest of a
# crate's rdeps still runs. rdeps(CRATE) includes CRATE itself.
[group('dev')]
[doc('Run the tests of CRATE and its reverse dependencies (no plugin/preview suites).')]
[unix]
t-worker CRATE:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --profile worker --workspace --all-targets --all-features --no-tests=pass -E 'rdeps({{ CRATE }})'

# scripts/touched-filter.sh maps each changed src file to its module's tests; pass FILES to override.
[group('dev')]
[doc('Run the tests of the modules touched by the current git diff.')]
[unix]
t-touched:
    #!/usr/bin/env bash
    set -euo pipefail
    filter="$(scripts/touched-filter.sh)"
    packages=$(grep -oE 'package\([^)]*\)' <<<"$filter" | sed -E 's/package\((.*)\)/-p \1/' | sort -u | tr '\n' ' ')
    if [ -z "$packages" ]; then echo "t-touched: nothing touched"; exit 0; fi
    TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run $packages --lib --bins --all-features --no-tests=pass -E "$filter"

[group('dev')]
[doc('Clippy plus lele-lint for one package (e.g. just check-crate starboard).')]
check-crate CRATE:
    #!/usr/bin/env bash
    set -euo pipefail
    case "{{ CRATE }}" in
        common_types) dir=crates/common_types ;;
        lobby_contract) dir=crates/lobby-room/freenet_contract ;;
        freenet-libp2p-bevy-core) dir=crates/freenet-libp2p-bevy-core ;;
        freenet-libp2p-bevy-lobby-room) dir=crates/lobby-room/bevy_plugin ;;
        lobby_room_shared_types) dir=crates/lobby-room/shared_types ;;
        freenet-libp2p-bevy-plugin) dir=crates/freenet-libp2p-bevy-plugin ;;
        lobby_room) dir=examples/lobby_room ;;
        starboard) dir=examples/starboard ;;
        input_fidelity_e2e) dir=tests/input-fidelity ;;
        lobby_room_e2e) dir=tests/lobby-room ;;
        starboard_e2e) dir=tests/starboard ;;
        *)
            echo "check-crate: unknown crate '{{ CRATE }}'; known: common_types lobby_contract freenet-libp2p-bevy-core freenet-libp2p-bevy-lobby-room lobby_room_shared_types freenet-libp2p-bevy-plugin lobby_room starboard input_fidelity_e2e lobby_room_e2e starboard_e2e" >&2
            exit 1
            ;;
    esac
    cargo clippy -p {{ CRATE }} --all-targets --all-features -- -D warnings
    lele-lint "$dir"

[group('dev')]
[doc('Check formatting (cargo fmt --all -- --check).')]
fmt:
    cargo fmt --all -- --check

[group('dev')]
[doc('Lint the whole workspace; warnings deny the gate.')]
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

[group('dev')]
[doc('Run lele-lint over every workspace crate dir.')]
lint:
    lele-lint --config crates/freenet-libp2p-bevy-core/lele.toml --sync-config --check crates/freenet-libp2p-bevy-core
    lele-lint --config crates/lobby-room/bevy_plugin/lele.toml --sync-config --check crates/lobby-room/bevy_plugin
    lele-lint --config crates/freenet-libp2p-bevy-plugin/lele.toml --sync-config --check crates/freenet-libp2p-bevy-plugin
    lele-lint --config crates/lobby-room/freenet_contract/lele.toml --sync-config --check crates/lobby-room/freenet_contract
    lele-lint --config crates/lobby-room/shared_types/lele.toml --sync-config --check crates/lobby-room/shared_types
    lele-lint --config crates/common_types/lele.toml --sync-config --check crates/common_types
    lele-lint crates/freenet-libp2p-bevy-core
    lele-lint crates/lobby-room/bevy_plugin
    lele-lint crates/freenet-libp2p-bevy-plugin
    lele-lint crates/lobby-room/freenet_contract
    lele-lint crates/lobby-room/shared_types
    lele-lint crates/common_types

[group('dev')]
[doc('Run lele-bevy-lint over the Bevy crates.')]
bevy-lint:
    lele-bevy-lint --config crates/freenet-libp2p-bevy-core/lele.toml --sync-config --check crates/freenet-libp2p-bevy-core
    lele-bevy-lint --config crates/lobby-room/bevy_plugin/lele.toml --sync-config --check crates/lobby-room/bevy_plugin
    lele-bevy-lint --config crates/freenet-libp2p-bevy-plugin/lele.toml --sync-config --check crates/freenet-libp2p-bevy-plugin
    lele-bevy-lint crates/freenet-libp2p-bevy-core
    lele-bevy-lint crates/lobby-room/bevy_plugin
    lele-bevy-lint crates/freenet-libp2p-bevy-plugin

# Regenerate the __methods__ indexes after adding or removing #[atomic_delegates] shells.
[group('dev')]
[doc('Regenerate __methods__ indexes after adding/removing atomic shells.')]
sync-methods:
    lele-lint --sync-methods crates/freenet-libp2p-bevy-core
    lele-lint --sync-methods crates/lobby-room/bevy_plugin
    lele-lint --sync-methods crates/lobby-room/freenet_contract
    lele-lint --sync-methods crates/lobby-room/shared_types

[group('gate')]
[doc('Run the full workspace test suite (nextest, lavapipe ICD).')]
test:
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --workspace --all-targets --all-features

# every feature combination users can pick must compile
[group('gate')]
[doc('Check every feature combination users can pick.')]
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

[group('gate')]
[doc('Check the wasm32 targets (common_types, lobby_room_shared_types).')]
wasm:
    cargo check -p common_types --target wasm32-unknown-unknown
    cargo check -p lobby_room_shared_types --target wasm32-unknown-unknown

[group('gate')]
[doc('Run the full prek gate on all files (same as CI).')]
check:
    prek run --all-files

[group('gate')]
[doc('Full gate: build, features, check.')]
ci: build features check

[group('gate')]
[doc('Install the prek git hooks (one-time per clone).')]
setup:
    prek install -t pre-commit -t pre-push

# UI layer on the fake network (BRP suite by default): fast, for reviewing the UI.
# Telegram stays silent. `just e2e direct_dial` runs one name pattern.
# The `[^:]*::[^:]*` conjunct keeps the e2e test fns (one `mod::fn` level) and
# leaves the unit tests nested inside the `e2e` binaries to `just test`.
[group('e2e')]
[doc('Run e2e BRP tests on the fake network (optional name pattern).')]
[linux]
e2e TEST="":
    #!/usr/bin/env bash
    set -euo pipefail
    VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=fake TELEGRAM_BOT_TOKEN= TELEGRAM_CHAT_ID= cargo nextest run -p lobby_room_e2e -p starboard_e2e --no-fail-fast -j 1 --test e2e -E 'test(/{{ TEST }}/) and test(/^[^:]*::[^:]*$/)' --all-features --run-ignored all -- --nocapture

# Windows e2e is not supported yet and windows will be on top; gdigrab recording comes later.
# E2E_WINDOW_MODE is not read by the harness yet.
[group('e2e')]
[doc('Run e2e BRP tests on the fake network (optional name pattern).')]
[windows]
e2e TEST="":
    $env:E2E_NETWORK="fake"; $env:TELEGRAM_BOT_TOKEN=""; $env:TELEGRAM_CHAT_ID=""; $env:E2E_WINDOW_MODE="on-top"; cargo nextest run -p lobby_room_e2e -p starboard_e2e --no-fail-fast -j 1 --test e2e -E 'test(/{{ TEST }}/) and test(/^[^:]*::[^:]*$/)' --all-features --run-ignored all -- --nocapture

[group('e2e')]
[doc('Run e2e BRP tests on the fake network (optional name pattern).')]
[macos]
e2e TEST="":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "e2e on macOS is not supported yet" >&2
    exit 1

# UI layer on the fake network, one recorded video per test sent to Telegram.
# Needs an Xvfb (E2E_XVFB=/path/to/Xvfb when it is not on PATH) and
# TELEGRAM_BOT_TOKEN + TELEGRAM_CHAT_ID (env or .env).
# Encode without sending: E2E_RECORD_DRY_RUN=1 just e2e-record
[group('e2e')]
[doc('Run e2e BRP tests on fake, recording one video per test.')]
[linux]
e2e-record TEST="":
    #!/usr/bin/env bash
    set -euo pipefail
    E2E_RECORD=1 VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=fake cargo nextest run -p lobby_room_e2e -p starboard_e2e --no-fail-fast -j 1 --test e2e -E 'test(/{{ TEST }}/) and test(/^[^:]*::[^:]*$/)' --all-features --run-ignored all -- --nocapture

# Windows e2e is not supported yet and windows will be on top; gdigrab recording comes later.
# E2E_WINDOW_MODE is not read by the harness yet.
[group('e2e')]
[doc('Run e2e BRP tests on fake, recording one video per test.')]
[windows]
e2e-record TEST="":
    $env:E2E_RECORD="1"; $env:E2E_NETWORK="fake"; $env:E2E_WINDOW_MODE="on-top"; cargo nextest run -p lobby_room_e2e -p starboard_e2e --no-fail-fast -j 1 --test e2e -E 'test(/{{ TEST }}/) and test(/^[^:]*::[^:]*$/)' --all-features --run-ignored all -- --nocapture

[group('e2e')]
[doc('Run e2e BRP tests on fake, recording one video per test.')]
[macos]
e2e-record TEST="":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "e2e on macOS is not supported yet" >&2
    exit 1

# The matching BRP tests on the public network, one at a time with PAUSE seconds
# between them (the 90 s gap lets the previous test's peers leave the ring).
# Repeats the whole selection RUNS times with PAUSE between runs (never before
# the first); counts how many runs had failures and exits 1 if any did.
# Records every test and sends its video to Telegram unless E2E_RECORD=0 / E2E_RECORD_DRY_RUN=1.
[group('e2e')]
[doc('Run e2e BRP tests on mainnet, RUNS times, PAUSE s apart.')]
[linux]
e2e-mainnet TEST="" PAUSE="90" RUNS="1":
    #!/usr/bin/env bash
    set -euo pipefail
    # LOBBY_RUST_LOG default is loop-only; an exported value still wins.
    export LOBBY_RUST_LOG="${LOBBY_RUST_LOG:-{{ e2e_log_filter }}}"
    export E2E_RECORD="${E2E_RECORD:-1}"
    names="$(cargo nextest list -p lobby_room_e2e -p starboard_e2e --test e2e --all-features --run-ignored all --message-format oneline -E 'test(/{{ TEST }}/) and test(/^[^:]*::[^:]*$/)' 2>/dev/null | sed -n 's/^lobby_room_e2e::e2e //p; s/^starboard_e2e::e2e //p')"
    if [ -z "$names" ]; then
        echo "e2e-mainnet: no test matches '{{ TEST }}'" >&2
        exit 1
    fi
    bad_runs=0
    for run in $(seq 1 {{ RUNS }}); do
        if [ "$run" -gt 1 ]; then sleep {{ PAUSE }}; fi
        echo "e2e-mainnet: run $run of {{ RUNS }}"
        first=1
        failed=()
        while IFS= read -r name; do
            if [ "$first" = 0 ]; then sleep {{ PAUSE }}; fi
            first=0
            echo "e2e-mainnet: $name"
            if ! VK_ICD_FILENAMES={{ _lavapipe }} E2E_NETWORK=mainnet cargo nextest run -p lobby_room_e2e -p starboard_e2e --no-fail-fast -j 1 --test e2e -E "test(=$name)" --all-features --run-ignored all -- --nocapture; then
                failed+=("$name")
            fi
        done <<< "$names"
        if [ "${#failed[@]}" -gt 0 ]; then
            echo "e2e-mainnet: run $run: ${#failed[@]} failed:" >&2
            printf '  %s\n' "${failed[@]}" >&2
            bad_runs=$((bad_runs + 1))
        else
            echo "e2e-mainnet: run $run passed"
        fi
    done
    echo "e2e-mainnet: $bad_runs of {{ RUNS }} runs had failures"
    [ "$bad_runs" = 0 ]

# Windows e2e is not supported yet and windows will be on top; gdigrab recording comes later.
# E2E_WINDOW_MODE is not read by the harness yet.
[group('e2e')]
[doc('Run e2e BRP tests on mainnet, RUNS times, PAUSE s apart.')]
[windows]
e2e-mainnet TEST="" PAUSE="90" RUNS="1":
    if (-not $env:LOBBY_RUST_LOG) { $env:LOBBY_RUST_LOG = '{{ e2e_log_filter }}' }
    if (-not $env:E2E_RECORD) { $env:E2E_RECORD = '1' }
    $env:E2E_WINDOW_MODE="on-top"; $names = cargo nextest list -p lobby_room_e2e -p starboard_e2e --test e2e --all-features --run-ignored all --message-format oneline -E 'test(/{{ TEST }}/) and test(/^[^:]*::[^:]*$/)' | ForEach-Object { $_ -replace '^lobby_room_e2e::e2e ', '' -replace '^starboard_e2e::e2e ', '' } | Where-Object { $_ -ne '' }
    if (-not $names) { Write-Host "e2e-mainnet: no test matches '{{ TEST }}'"; exit 1 }
    $bad_runs = 0
    for ($run = 1; $run -le {{ RUNS }}; $run++) { if ($run -gt 1) { Start-Sleep -Seconds {{ PAUSE }} }; Write-Host "e2e-mainnet: run $run of {{ RUNS }}"; $first = 1; $failed = @(); foreach ($name in $names) { if ($first -eq 0) { Start-Sleep -Seconds {{ PAUSE }} }; $first = 0; Write-Host "e2e-mainnet: $name"; $env:E2E_NETWORK="mainnet"; cargo nextest run -p lobby_room_e2e -p starboard_e2e --no-fail-fast -j 1 --test e2e -E "test(=$name)" --all-features --run-ignored all -- --nocapture; if ($LASTEXITCODE -ne 0) { $failed += $name } }; if ($failed.Count -gt 0) { Write-Host "e2e-mainnet: run ${run}: $($failed.Count) failed"; $failed | ForEach-Object { Write-Host "  $_" }; $bad_runs++ } else { Write-Host "e2e-mainnet: run $run passed" } }
    Write-Host "e2e-mainnet: $bad_runs of {{ RUNS }} runs had failures"
    if ($bad_runs -ne 0) { exit 1 }

[group('e2e')]
[doc('Run e2e BRP tests on mainnet, RUNS times, PAUSE s apart.')]
[macos]
e2e-mainnet TEST="" PAUSE="90" RUNS="1":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "e2e on macOS is not supported yet" >&2
    exit 1

# Input-fidelity e2e: proves the synthetic e2e input writes the same Bevy input
# messages as real XTest input. Starts its own Xvfb (never :0); prints the diff
# when the two logs disagree. Repeat and SameFrame stay synthetic-only (see SYNTHETIC_ONLY_SCRIPT).
[group('e2e')]
[doc('Prove synthetic e2e input matches real XTest input.')]
[linux]
input-fidelity TEST="synthetic_input_matches_real_os_input":
    #!/usr/bin/env bash
    set -euo pipefail
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run -p input_fidelity_e2e --test e2e --all-features --run-ignored all -j 1 -E 'test(/{{ TEST }}/)' -- --nocapture

[group('e2e')]
[doc('Prove synthetic e2e input matches real XTest input.')]
[windows]
input-fidelity TEST="synthetic_input_matches_real_os_input":
    Write-Host "input-fidelity is Linux-only (XTest)"; exit 1

[group('e2e')]
[doc('Prove synthetic e2e input matches real XTest input.')]
[macos]
input-fidelity TEST="synthetic_input_matches_real_os_input":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "input-fidelity is Linux-only (XTest)" >&2
    exit 1

# Contract CRDT-law suite (freenet-contract-harness) against the lobby contract.
[group('e2e')]
[doc('Run the contract CRDT-law suite against the lobby contract.')]
freenet-harness:
    cargo nextest run -p lobby_room_e2e --test freenet_harness --all-features --run-ignored all -- --nocapture

[group('demo')]
[doc('Run the lobby_room example detached (defaults: --username lele --ui default).')]
[unix]
lobby-room-demo *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{ ARGS }}" ]; then just _detached lobby_room {{ ARGS }}; else just _detached lobby_room --username lele --ui default; fi

[group('demo')]
[doc('Run the lobby_room example detached (defaults: --username lele --ui default).')]
[windows]
lobby-room-demo *ARGS:
    if ('{{ ARGS }}' -ne '') { just _detached lobby_room {{ ARGS }} } else { just _detached lobby_room --username lele --ui default }

[group('demo')]
[doc('Run the starboard example detached (default: --username lele).')]
[unix]
starboard-demo *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{ ARGS }}" ]; then just _detached starboard {{ ARGS }}; else just _detached starboard --username lele; fi

[group('demo')]
[doc('Run the starboard example detached (default: --username lele).')]
[windows]
starboard-demo *ARGS:
    if ('{{ ARGS }}' -ne '') { just _detached starboard {{ ARGS }} } else { just _detached starboard --username lele }

[private]
[unix]
_detached BIN *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -p {{ BIN }}
    mkdir -p .local-run
    log=".local-run/{{ BIN }}.log"
    setsid "$CARGO_TARGET_DIR/debug/{{ BIN }}" {{ ARGS }} >"$log" 2>&1 &
    echo "{{ BIN }} running detached - log: $log"

[private]
[windows]
_detached BIN *ARGS:
    cargo build -p {{ BIN }}
    New-Item -ItemType Directory -Force -Path .local-run | Out-Null
    $log = ".local-run/{{ BIN }}.log"
    Start-Process -FilePath "$env:CARGO_TARGET_DIR/debug/{{ BIN }}.exe" -ArgumentList '{{ ARGS }}' -RedirectStandardOutput $log -RedirectStandardError $log
    Write-Host "{{ BIN }} running detached - log: $log"

[group('previews')]
[doc('Render UI previews; pass --resend to re-send every preview to Telegram.')]
previews *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ " {{ FLAGS }} " == *" --resend "* ]]; then export LELE_PREVIEW_RESEND_ALL=1; fi
    VK_ICD_FILENAMES={{ _lavapipe }} cargo nextest run --workspace --all-targets --all-features -E 'test(/_preview$/)'

# The only way to rebuild the lobby contract wasm. build.rs was removed because it
# inherited RUSTC_WORKSPACE_WRAPPER=clippy-driver from `cargo clippy`, which rebuilt
# the same source to different bytes. This recipe clears the wrappers so the output
# is repeatable, then copies the result over the canonical committed file.
[group('tools')]
[doc('Rebuild the lobby contract wasm (repeatable bytes) and refresh the committed file.')]
contract:
    #!/usr/bin/env bash
    set -euo pipefail
    env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u CLIPPY_ARGS cargo build -p lobby_contract --release --target wasm32-unknown-unknown
    cp "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/lobby_contract.wasm" crates/lobby-room/freenet_contract/lobby_contract.wasm
    wc -c crates/lobby-room/freenet_contract/lobby_contract.wasm
    sha256sum crates/lobby-room/freenet_contract/lobby_contract.wasm

[group('tools')]
[doc('Print a fresh game id (lobby/token namespace).')]
new-game-id:
    cargo run -p freenet-libp2p-bevy-core --bin new-game-id
