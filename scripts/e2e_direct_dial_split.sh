#!/usr/bin/env bash
# Local proof for Phase 6: runs direct_dial and friends_reconnect with the two
# peers split across two Orchestrator processes on this one machine (the LOCAL
# board), then runs E2E_ROLE=check over the merged results.
# Usage: scripts/e2e_direct_dial_split.sh [direct_dial|friends_reconnect|all]
set -euo pipefail

export TELEGRAM_BOT_TOKEN=
export TELEGRAM_CHAT_ID=
export LELE_PREVIEW_DRY_RUN=1
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/cargo-target/freenet-libp2p-bevy-plugin}"
export DISPLAY="${DISPLAY:-:0}"
export XAUTHORITY="${XAUTHORITY:-$HOME/.Xauthority}"
if [ -z "${VK_ICD_FILENAMES:-}" ]; then
    VK_ICD_FILENAMES=$(ls /usr/share/vulkan/icd.d/lvp_icd*.json 2>/dev/null | head -n1 || true)
    export VK_ICD_FILENAMES
fi

wanted="${1:-all}"

cargo build -p freenet-libp2p-bevy-plugin --features e2e --example starboard
TEST_BIN=$(cargo test -p e2e --test e2e --no-run 2>&1 | tee /dev/stderr | sed -n 's/^ *Executable .*tests\/e2e\/main\.rs (\(.*\))$/\1/p' | head -n1)
if [ ! -x "$TEST_BIN" ]; then
    echo "e2e test binary not found" >&2
    exit 1
fi
GAME_BIN="$CARGO_TARGET_DIR/debug/examples/starboard"
echo "game: $GAME_BIN"
echo "test: $TEST_BIN"

ROOT=$(mktemp -d)
echo "results: $ROOT"

# The harness kills stray games on exit; the game command line contains the
# example path, so this only ever matches game processes.
trap 'pkill -f "examples/starboard" 2>/dev/null || true' EXIT

run_split() {
    local test=$1
    echo "=== $test: starting a/b ==="
    # Each test gets its own board: peer entries carry the run's peer ids
    # and ports, so reusing one board across tests would substitute the
    # previous test's stale addresses into the next test's scripts.
    local board="$ROOT/$test/board"
    mkdir -p "$board"
    echo "board: $board"
    E2E_MACHINE=a E2E_PLACEMENT="alice=a,bob=b" E2E_BOARD_DIR="$board" E2E_GAME_BIN="$GAME_BIN" E2E_RUN_DIR="$ROOT/$test/a" \
        "$TEST_BIN" "$test" --ignored --nocapture > "$ROOT/$test-a.log" 2>&1 &
    local a_pid=$!
    E2E_MACHINE=b E2E_PLACEMENT="alice=a,bob=b" E2E_BOARD_DIR="$board" E2E_GAME_BIN="$GAME_BIN" E2E_RUN_DIR="$ROOT/$test/b" \
        "$TEST_BIN" "$test" --ignored --nocapture > "$ROOT/$test-b.log" 2>&1 &
    local b_pid=$!
    local a_status=0 b_status=0
    wait "$a_pid" || a_status=$?
    wait "$b_pid" || b_status=$?
    echo "=== $test: machine a exit $a_status, machine b exit $b_status ==="
    if [ "$a_status" -ne 0 ] || [ "$b_status" -ne 0 ]; then
        return 1
    fi
    local merged="$ROOT/$test/merged"
    mkdir -p "$merged"
    cp -r "$ROOT/$test/a/." "$merged/"
    cp -r "$ROOT/$test/b/." "$merged/"
    echo "=== $test: check role over $merged ==="
    E2E_ROLE=check E2E_RESULTS_DIR="$merged" "$TEST_BIN" "$test" --ignored --nocapture
}

failed=0
if [ "$wanted" = "all" ] || [ "$wanted" = "direct_dial" ]; then
    run_split direct_dial || failed=1
fi
if [ "$wanted" = "all" ] || [ "$wanted" = "friends_reconnect" ]; then
    run_split friends_reconnect || failed=1
fi
if [ "$failed" -ne 0 ]; then
    echo "SPLIT RUN FAILED (logs in $ROOT)"
    exit 1
fi
echo "SPLIT RUN PASSED (results in $ROOT)"
