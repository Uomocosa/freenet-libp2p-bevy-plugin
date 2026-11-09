#!/usr/bin/env bash
# Prints one cross-OS scenario as KEY=value lines for $GITHUB_OUTPUT: the --script of
# each peer (w on Windows, l1 and l2 on Linux) and the e2e_check expectations.
# Usage: e2e_cross_scenario.sh <scenario> <time-scale> <room>
#
# Every @Ns deadline is multiplied by <time-scale>: the game only scales its lease and
# history timings by E2E_TIME_SCALE, so the script's own waits are scaled here.
set -euo pipefail

scenario=$1
scale=$2
room=$3

# `count` strokes of the scripted session, as in strokes() of tests/e2e/tests/common/scenario.rs.
strokes() {
    local count=$1 verb=$2 index x y
    for ((index = 0; index < count; index++)); do
        x=$((200 + index * 30))
        y=$((300 + index * 9))
        printf '%s:%s.%s-%s.%s,' "$verb" "$x" "$y" "$((x + 60))" "$y"
    done
}

scaled() {
    SCALE=$scale perl -pe 's/@(\d+)s/"@" . int($1 * $ENV{SCALE} + 0.5) . "s"/ge'
}

case "$scenario" in
s1)
    # Scenario::starboard(3): W paints 6, L1 erases 3, L2 watches.
    create="create:$room,wait-join-succeeded:$room@180s"
    join="wait-lobby-room:$room@420s,join:$room,wait-join-succeeded:$room@420s"
    members="wait-members:3@420s"
    end="wait-canvas-stable@60s,sleep:10s,exit-app"
    seen="wait-canvas-nonempty@120s"
    script_w="$create,$members,$(strokes 6 paint-from-to)$end"
    script_l1="$join,$members,$seen,$(strokes 3 erase-from-to)$end"
    script_l2="$join,$members,$seen,$end"
    expect="converge min-members:3"
    ;;
*)
    echo "unknown scenario '$scenario'; only s1 is ported so far" >&2
    exit 2
    ;;
esac

echo "script_w=$(scaled <<<"$script_w")"
echo "script_l1=$(scaled <<<"$script_l1")"
echo "script_l2=$(scaled <<<"$script_l2")"
echo "expect=$expect"
