#!/usr/bin/env python3
"""Compare the two newest dry_run preview runs for hash stability.

Reads ui_preview/preview-log.jsonl (path given as argv[1]), takes the two
newest distinct run ids with outcome == "dry_run" in log order, and compares
new_hash per key across (and within) those runs.

Exit 0 when stable, 1 with a key-by-key report when unstable.
"""

import json
import sys
from collections import OrderedDict


def load_entries(log_path):
    entries = []
    with open(log_path, "r", encoding="utf-8") as handle:
        for lineno, line in enumerate(handle, start=1):
            line = line.strip()
            if not line:
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError as exc:
                print(f"unparsable line {lineno}: {exc}")
                continue
            entries.append(record)
    return entries


def main(argv):
    if len(argv) != 2:
        print(f"usage: {argv[0]} <preview-log.jsonl>")
        return 2
    log_path = argv[1]
    try:
        entries = load_entries(log_path)
    except FileNotFoundError:
        print(f"preview-stability: log not found: {log_path}")
        return 1
    except OSError as exc:
        print(f"preview-stability: cannot read {log_path}: {exc}")
        return 1

    dry = [e for e in entries if e.get("outcome") == "dry_run"]
    if not dry:
        print("preview-stability: no entries with outcome dry_run in log")
        return 1

    ordered_runs = list(OrderedDict.fromkeys(e.get("run", "") for e in dry))
    ordered_runs = [r for r in ordered_runs if r]
    if len(ordered_runs) < 2:
        print(
            "preview-stability: need two distinct dry_run run ids, "
            f"found {len(ordered_runs)}"
        )
        return 1
    run_a, run_b = ordered_runs[-2], ordered_runs[-1]

    entries_a = [e for e in dry if e.get("run") == run_a]
    entries_b = [e for e in dry if e.get("run") == run_b]

    def hashes_by_key(rows):
        grouped = {}
        for row in rows:
            key = row.get("key", "")
            grouped.setdefault(key, set()).add(row.get("new_hash", ""))
        return grouped

    hashes_a = hashes_by_key(entries_a)
    hashes_b = hashes_by_key(entries_b)

    problems = []
    for key in sorted(set(hashes_a)):
        if len(hashes_a[key]) > 1:
            problems.append(
                f"intra-run disagreement in {run_a}: {key} "
                f"hashes {sorted(hashes_a[key])}"
            )
    for key in sorted(set(hashes_b)):
        if len(hashes_b[key]) > 1:
            problems.append(
                f"intra-run disagreement in {run_b}: {key} "
                f"hashes {sorted(hashes_b[key])}"
            )

    all_keys = sorted(set(hashes_a) | set(hashes_b))
    for key in all_keys:
        if key not in hashes_a:
            problems.append(f"missing in run {run_a}: {key} (B={sorted(hashes_b[key])})")
        elif key not in hashes_b:
            problems.append(f"missing in run {run_b}: {key} (A={sorted(hashes_a[key])})")
        else:
            hash_a = next(iter(hashes_a[key]))
            hash_b = next(iter(hashes_b[key]))
            if len(hashes_a[key]) == 1 and len(hashes_b[key]) == 1 and hash_a != hash_b:
                problems.append(f"{key}: A={hash_a} B={hash_b}")

    non_same_keys = set()
    for row in entries_a + entries_b:
        if row.get("status") != "same":
            non_same_keys.add(row.get("key", ""))
    print(
        f"non-same statuses: {len(non_same_keys)} of {len(all_keys)} keys "
        f"have status other than 'same' in runs {run_a} and {run_b}"
    )

    if problems:
        print(f"unstable keys ({len(problems)}):")
        for item in problems:
            print(f"  {item}")
        print(
            f"preview-stability: UNSTABLE compared {len(all_keys)} keys "
            f"between {run_a} and {run_b}: "
            f"{len(problems)} differing/missing/intra-run conflicts, "
            f"{len(non_same_keys)} non-same statuses"
        )
        return 1

    print(
        f"preview-stability: STABLE compared {len(all_keys)} keys "
        f"between {run_a} and {run_b}, "
        f"{len(non_same_keys)} non-same statuses"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
