#!/usr/bin/env python3
"""Per-peer timing stats for freenet_ring e2e runs (stdlib only).

Reads .local-run/<scenario>-<epoch>/instance-pN.log and reports, per peer,
seconds elapsed from that log's first timestamped line:

  gate_open_s         first "lobby freenet node reachable"
                      (or old "lobby freenet ring ready")
  gave_up             "lobby freenet node unreachable, proceeding anyway" appears
                      (or old "ring not ready, proceeding via gateway")
  ring_join_s         first "connect driver: joined peer"
  first_ring_peers_s  first line with ring_peers=N, N > 0
  lobby_ready_s       first "lobby ready key="
  fetch               elapsed_ms + outcome of the first
                      "lobby fetch found" / "lobby fetch failed"
  deployed            "lobby contract deployed" appears
  cached_reconnect    "Attempting fast reconnection" appears
  listen_port         port from "Node will be listening at :::port"
  max_ring_peers      max ring_peers=N seen

Per run, on one clock (the earliest first timestamp across the run's logs):

  room_created_s      earliest any peer logs a lobby with rooms>=1
                      ("lobby changed", "lobby ready" or "lobby fetch found")
  all_saw_room_s      when the LAST peer first shows rooms>=1;
                      missing ("-") if some peer never did
  deployers           number of peers with "lobby contract deployed"

The aggregate adds all_saw_room_s, room_lag_s (= all_saw_room_s -
room_created_s), and the per-run deployers counts.

Usage:
  scripts/ring_stats.py [--scenario NAME] [--last K] [--since EPOCH] [--json FILE] [run_dir ...]

  --scenario NAME  run directory prefix (default: freenet_ring)
  --last K         the K newest runs by epoch (default: 5)
  --since EPOCH    only run directories with epoch >= EPOCH;
                   overrides the --last default (keeps all matching runs)
  explicit run directories (relative or absolute) override --scenario/--last/--since

Missing values print as "-" (null in JSON). Truncated logs and runs with
no logs never crash the script; they show up as "-" rows / empty runs.
"""

import argparse
import datetime
import glob
import json
import math
import os
import re
import sys

TIMESTAMP_RE = re.compile(
    r"^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z)"
)
RING_PEERS_RE = re.compile(r"ring_peers=(\d+)")
ELAPSED_MS_RE = re.compile(r"elapsed_ms=(\d+)")
ROOMS_RE = re.compile(r"rooms=(\d+)")
KEY_RE = re.compile(r"key=([A-Za-z0-9_-]+)")
LISTEN_PORT_RE = re.compile(r"Node will be listening at .*?:(\d+) internal address")

GATE_OPEN_OLD = "lobby freenet ring ready"
GATE_OPEN_NEW = "lobby freenet node reachable"
GAVE_UP_OLD = "ring not ready, proceeding via gateway"
GAVE_UP_NEW = "lobby freenet node unreachable, proceeding anyway"
RING_JOINED = "connect driver: joined peer"
LOBBY_READY = "lobby ready key="
LOBBY_CHANGED = "lobby changed"
FETCH_FOUND = "lobby fetch found"
FETCH_FAILED = "lobby fetch failed"
DEPLOYED = "lobby contract deployed"
FAST_RECONNECT = "Attempting fast reconnection"


def parse_timestamp(line):
    match = TIMESTAMP_RE.match(line)
    if not match:
        return None
    text = match.group(1)
    try:
        if "." in text:
            dt = datetime.datetime.strptime(text, "%Y-%m-%dT%H:%M:%S.%fZ")
        else:
            dt = datetime.datetime.strptime(text, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError:
        return None
    return dt.replace(tzinfo=datetime.timezone.utc)


def parse_peer_log(path):
    """Parse one instance-pN.log. Never raises; missing data stays None."""
    peer = {
        "gate_open_s": None,
        "gave_up": False,
        "ring_join_s": None,
        "first_ring_peers_s": None,
        "lobby_ready_s": None,
        "fetch_ms": None,
        "fetch_outcome": None,
        "deployed": False,
        "cached_reconnect": False,
        "listen_port": None,
        "max_ring_peers": None,
        "keys": [],
        "t0_epoch": None,
        "first_room_epoch": None,
    }
    try:
        with open(path, errors="replace") as handle:
            lines = handle.readlines()
    except OSError:
        return peer
    t0 = None
    for line in lines:
        if t0 is None:
            t0 = parse_timestamp(line)
            if t0 is None:
                continue
            peer["t0_epoch"] = t0.timestamp()
        stamp = parse_timestamp(line)
        if stamp is None:
            continue
        elapsed = (stamp - t0).total_seconds()
        if (GATE_OPEN_OLD in line or GATE_OPEN_NEW in line) and peer["gate_open_s"] is None:
            peer["gate_open_s"] = elapsed
        if GAVE_UP_OLD in line or GAVE_UP_NEW in line:
            peer["gave_up"] = True
        if RING_JOINED in line and peer["ring_join_s"] is None:
            peer["ring_join_s"] = elapsed
        if peer["first_ring_peers_s"] is None:
            for value in RING_PEERS_RE.findall(line):
                if int(value) > 0:
                    peer["first_ring_peers_s"] = elapsed
                    break
        if LOBBY_READY in line and peer["lobby_ready_s"] is None:
            peer["lobby_ready_s"] = elapsed
        if peer["fetch_outcome"] is None:
            if FETCH_FOUND in line or FETCH_FAILED in line:
                outcome = "found" if FETCH_FOUND in line else "failed"
                elapsed_ms = ELAPSED_MS_RE.search(line)
                peer["fetch_outcome"] = outcome
                peer["fetch_ms"] = (
                    int(elapsed_ms.group(1)) if elapsed_ms else None
                )
        if peer["first_room_epoch"] is None and (
            LOBBY_CHANGED in line or LOBBY_READY in line or FETCH_FOUND in line
        ):
            for value in ROOMS_RE.findall(line):
                if int(value) >= 1:
                    peer["first_room_epoch"] = stamp.timestamp()
                    break
        if DEPLOYED in line:
            peer["deployed"] = True
        if FAST_RECONNECT in line:
            peer["cached_reconnect"] = True
        if peer["listen_port"] is None:
            found = LISTEN_PORT_RE.search(line)
            if found:
                peer["listen_port"] = int(found.group(1))
        for value in RING_PEERS_RE.findall(line):
            count = int(value)
            if peer["max_ring_peers"] is None or count > peer["max_ring_peers"]:
                peer["max_ring_peers"] = count
    return peer


def peer_name(path):
    base = os.path.basename(path)
    match = re.fullmatch(r"instance-(p\d+)\.log", base)
    return match.group(1) if match else base


def parse_run(run_dir):
    """Parse one .local-run/<scenario>-<epoch> directory. Never raises."""
    peers = []
    try:
        logs = sorted(glob.glob(os.path.join(run_dir, "instance-p*.log")))
    except OSError:
        logs = []
    for path in logs:
        peer = parse_peer_log(path)
        peer["peer"] = peer_name(path)
        keys = set()
        try:
            with open(path, errors="replace") as handle:
                for line in handle:
                    if LOBBY_READY in line or DEPLOYED in line:
                        keys.update(KEY_RE.findall(line))
        except OSError:
            pass
        peer["keys"] = sorted(keys)
        peers.append(peer)
    keys = sorted({key for peer in peers for key in peer["keys"]})
    t0s = [peer["t0_epoch"] for peer in peers if peer["t0_epoch"] is not None]
    run_t0 = min(t0s) if t0s else None
    room_times = sorted(
        peer["first_room_epoch"] for peer in peers if peer["first_room_epoch"] is not None
    )
    if run_t0 is not None and room_times:
        room_created_s = room_times[0] - run_t0
    else:
        room_created_s = None
    if run_t0 is not None and peers and len(room_times) == len(peers):
        all_saw_room_s = room_times[-1] - run_t0
    else:
        all_saw_room_s = None
    return {
        "run": os.path.basename(os.path.normpath(run_dir)),
        "peers": peers,
        "keys": keys,
        "gave_up": [p["peer"] for p in peers if p["gave_up"]],
        "deployed": [p["peer"] for p in peers if p["deployed"]],
        "deployers": sum(1 for p in peers if p["deployed"]),
        "room_created_s": room_created_s,
        "all_saw_room_s": all_saw_room_s,
    }


def percentile_nearest_rank(values, percent):
    """Nearest-rank percentile over a non-empty list of numbers."""
    ordered = sorted(values)
    rank = max(1, math.ceil(percent / 100.0 * len(ordered)))
    return ordered[rank - 1]


def summarize(values):
    if not values:
        return {"n": 0, "p50": None, "p90": None, "max": None}
    return {
        "n": len(values),
        "p50": percentile_nearest_rank(values, 50),
        "p90": percentile_nearest_rank(values, 90),
        "max": max(values),
    }


def aggregate(runs):
    peers = [peer for run in runs for peer in run["peers"]]
    room_lags = [
        run["all_saw_room_s"] - run["room_created_s"]
        for run in runs
        if run["all_saw_room_s"] is not None and run["room_created_s"] is not None
    ]
    result = {
        "runs": len(runs),
        "peers": len(peers),
        "gate_open_s": summarize(
            [p["gate_open_s"] for p in peers if p["gate_open_s"] is not None]
        ),
        "ring_join_s": summarize(
            [p["ring_join_s"] for p in peers if p["ring_join_s"] is not None]
        ),
        "first_ring_peers_s": summarize(
            [
                p["first_ring_peers_s"]
                for p in peers
                if p["first_ring_peers_s"] is not None
            ]
        ),
        "lobby_ready_s": summarize(
            [p["lobby_ready_s"] for p in peers if p["lobby_ready_s"] is not None]
        ),
        "all_saw_room_s": summarize(
            [r["all_saw_room_s"] for r in runs if r["all_saw_room_s"] is not None]
        ),
        "room_lag_s": summarize(room_lags),
        "deployers": [run["deployers"] for run in runs],
        "gave_up_peers": sum(1 for p in peers if p["gave_up"]),
        "peers_no_lobby_ready": sum(1 for p in peers if p["lobby_ready_s"] is None),
        "runs_with_2plus_deployers": sorted(
            run["run"] for run in runs if len(run["deployed"]) >= 2
        ),
    }
    return result


def select_runs(scenario, last, since=None):
    """Run dirs for a scenario, newest first-filtered.

    Keeps only `.local-run/<scenario>-<epoch>` dirs whose numeric epoch is
    >= `since` (when given; non-numeric suffixes never match a bound), then
    the `last` newest of those (`last=None` keeps all).
    """
    prefix = scenario + "-"
    candidates = []
    for entry in glob.glob(os.path.join(".local-run", prefix + "*")):
        if not os.path.isdir(entry):
            continue
        name = os.path.basename(entry)
        epoch_text = name[len(prefix):]
        try:
            epoch = int(epoch_text)
        except ValueError:
            epoch = None
        if since is not None and (epoch is None or epoch < since):
            continue
        order = epoch if epoch is not None else epoch_text
        candidates.append((order, entry))
    candidates.sort(key=lambda item: (str(type(item[0])), item[0]))
    if last is not None:
        if last <= 0:
            return []
        candidates = candidates[-last:]
    return [entry for _, entry in candidates]


def fmt_seconds(value):
    return "-" if value is None else f"{value:.1f}"


def fmt_fetch(peer):
    if peer["fetch_outcome"] is None:
        return "-"
    ms = "-" if peer["fetch_ms"] is None else str(peer["fetch_ms"])
    return f"{ms}/{peer['fetch_outcome']}"


def print_table(runs, stats, empty_note=None):
    if runs:
        print(f"ring stats: scenario runs={len(runs)}")
    elif empty_note:
        print(f"ring stats: no runs selected ({empty_note})")
    else:
        print("ring stats: no runs selected")
    for run in runs:
        keys = ",".join(run["keys"]) if run["keys"] else "-"
        print(f"\nrun {run['run']} peers={len(run['peers'])} keys=[{keys}]")
        if not run["peers"]:
            print("  (no instance-pN.log files)")
            continue
        print(
            "  peer  gate_open_s   gave_up  "
            "ring_join_s  first_ring_peers_s  lobby_ready_s  fetch        deployed  max_ring_peers  cached_reconnect  listen_port"
        )
        for peer in run["peers"]:
            max_ring = (
                "-" if peer["max_ring_peers"] is None else str(peer["max_ring_peers"])
            )
            listen_port = (
                "-" if peer["listen_port"] is None else str(peer["listen_port"])
            )
            print(
                f"  {peer['peer']:<5}"
                f"{fmt_seconds(peer['gate_open_s']):<14}"
                f"{'yes' if peer['gave_up'] else 'no':<9}"
                f"{fmt_seconds(peer['ring_join_s']):<13}"
                f"{fmt_seconds(peer['first_ring_peers_s']):<20}"
                f"{fmt_seconds(peer['lobby_ready_s']):<15}"
                f"{fmt_fetch(peer):<13}"
                f"{'yes' if peer['deployed'] else 'no':<10}"
                f"{max_ring:<16}"
                f"{'yes' if peer['cached_reconnect'] else 'no':<18}"
                f"{listen_port}"
            )
        gave_up = ",".join(run["gave_up"]) if run["gave_up"] else "-"
        deployed = ",".join(run["deployed"]) if run["deployed"] else "-"
        print(f"  gave_up: {gave_up}  deployed: {deployed}")
        print(
            f"  room_created_s: {fmt_seconds(run['room_created_s'])}"
            f"  all_saw_room_s: {fmt_seconds(run['all_saw_room_s'])}"
            f"  deployers: {run['deployers']}"
        )
    print(f"\naggregate over {stats['runs']} runs ({stats['peers']} peers):")
    print("  metric              n    p50    p90    max")
    for metric in ("gate_open_s", "ring_join_s", "first_ring_peers_s", "lobby_ready_s",
                   "all_saw_room_s", "room_lag_s"):
        summary = stats[metric]
        print(
            f"  {metric:<18}"
            f"{summary['n']:<5}"
            f"{fmt_seconds(summary['p50']):<7}"
            f"{fmt_seconds(summary['p90']):<7}"
            f"{fmt_seconds(summary['max'])}"
        )
    print(f"  deployers per run: {stats['deployers']}")
    multi = stats["runs_with_2plus_deployers"]
    print(f"  gave_up peers: {stats['gave_up_peers']}")
    print(f"  peers with no lobby_ready_s: {stats['peers_no_lobby_ready']}")
    print(f"  runs with 2+ deployers: {len(multi)}"
          + (f" [{','.join(multi)}]" if multi else ""))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", default="freenet_ring")
    parser.add_argument("--last", type=int, default=None)
    parser.add_argument(
        "--since",
        type=int,
        default=None,
        help="only .local-run/<scenario>-<epoch> dirs with epoch >= SINCE",
    )
    parser.add_argument("--json", default=None)
    parser.add_argument("run_dirs", nargs="*")
    args = parser.parse_args(argv)
    if args.run_dirs:
        # Explicit directories bypass --scenario/--last/--since selection.
        run_dirs = args.run_dirs
        empty_note = None
    else:
        if args.last is None and args.since is None:
            last = 5
        else:
            last = args.last
        run_dirs = select_runs(args.scenario, last, args.since)
        if not run_dirs:
            if args.since is not None:
                empty_note = f"no runs with epoch >= {args.since}"
                print(
                    f"no runs with epoch >= {args.since} for scenario "
                    f"{args.scenario!r} under .local-run/ (no stale data shown)",
                    file=sys.stderr,
                )
            else:
                empty_note = None
                print(
                    f"no runs match scenario {args.scenario!r} under .local-run/",
                    file=sys.stderr,
                )
        else:
            empty_note = None
    runs = [parse_run(run_dir) for run_dir in run_dirs]
    stats = aggregate(runs)
    print_table(runs, stats, empty_note)
    if args.json:
        with open(args.json, "w") as handle:
            json.dump({"runs": runs, "aggregate": stats}, handle, indent=2)
            handle.write("\n")
        print(f"\nwrote {args.json}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
