#!/usr/bin/env python3
"""Compare the newest N dry_run preview runs for hash stability.

Reads ui_preview/preview-log.jsonl (path given as argv[1]), takes the N
newest distinct run ids with outcome == "dry_run" in log order (N given as
argv[2], default 5), and compares new_hash per key across (and within)
those runs.

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
    if len(argv) not in (2, 3):
        print(f"usage: {argv[0]} <preview-log.jsonl> [N]")
        return 2
    log_path = argv[1]
    try:
        want = int(argv[2]) if len(argv) == 3 else 5
    except ValueError:
        print(f"preview-stability: invalid N: {argv[2]!r}")
        return 2
    if want < 2:
        print(f"preview-stability: N must be >= 2, got {want}")
        return 2
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
    if len(ordered_runs) < want:
        print(
            f"preview-stability: need {want} distinct dry_run run ids, "
            f"found {len(ordered_runs)}"
        )
        return 1
    runs = ordered_runs[-want:]

    def hashes_by_key(rows):
        grouped = {}
        for row in rows:
            key = row.get("key", "")
            grouped.setdefault(key, set()).add(row.get("new_hash", ""))
        return grouped

    by_run = {}
    for run in runs:
        by_run[run] = hashes_by_key([e for e in dry if e.get("run") == run])

    problems = []
    for run in runs:
        for key in sorted(by_run[run]):
            if len(by_run[run][key]) > 1:
                problems.append(
                    f"intra-run disagreement in {run}: {key} "
                    f"hashes {sorted(by_run[run][key])}"
                )

    all_keys = sorted({key for hashes in by_run.values() for key in hashes})
    for key in all_keys:
        missing = [run for run in runs if key not in by_run[run]]
        if missing:
            present = {run: sorted(by_run[run][key]) for run in runs if key in by_run[run]}
            problems.append(f"missing {key} in run(s) {missing} (present={present})")
            continue
        distinct = {next(iter(by_run[run][key])) for run in runs}
        single = all(len(by_run[run][key]) == 1 for run in runs)
        if single and len(distinct) > 1:
            detail = {run: next(iter(by_run[run][key])) for run in runs}
            problems.append(f"{key}: differing hashes {detail}")

    non_same_keys = set()
    for run in runs:
        for row in (e for e in dry if e.get("run") == run):
            if row.get("status") != "same":
                non_same_keys.add(row.get("key", ""))
    print(
        f"non-same statuses: {len(non_same_keys)} of {len(all_keys)} keys "
        f"have status other than 'same' in runs {', '.join(runs)}"
    )

    if problems:
        print(f"unstable keys ({len(problems)}):")
        for item in problems:
            print(f"  {item}")
        print(
            f"preview-stability: UNSTABLE compared {len(all_keys)} keys "
            f"across {len(runs)} runs {', '.join(runs)}: "
            f"{len(problems)} differing/missing/intra-run conflicts, "
            f"{len(non_same_keys)} non-same statuses"
        )
        return 1

    print(
        f"preview-stability: STABLE compared {len(all_keys)} keys "
        f"across {len(runs)} runs {', '.join(runs)}, "
        f"{len(non_same_keys)} non-same statuses"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
