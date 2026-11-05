#!/usr/bin/env python3
"""List (default) or delete (`apply`) orphaned UI preview artifacts.

Usage: preview_clean.py <ui_preview_dir> [apply]

Reads <ui_preview_dir>/preview-log.jsonl, takes the newest run id (last
non-empty line), and compares the files on disk against it:

- live keys: non-frame `scene|label` / `scene|clip:label` keys in the newest
  run, mapped to `<scene>__<slug(label)>.png` / `<scene>__<slug(label)>.mp4`
  (the `clip:` prefix is stripped for the file name).
- orphan previews: `*.png` / `*.mp4` on disk not in the live set.
- orphan `.sent` files: `_cache/*.sent` whose preview file is absent.
- stale frames dirs: `*_frames` whose scene is not live.

Default prints only. `apply` deletes orphans, orphan `.sent` files and
stale frames dirs. Never deletes `preview-log.jsonl`, `*.lock`, or frame
folders of live scenes.
"""

import json
import shutil
import sys
from pathlib import Path


def slug(label):
    return "".join(
        character.lower() if character.isascii() and character.isalnum() else "_"
        for character in label
    )


def load_entries(log_path):
    entries = []
    with open(log_path, "r", encoding="utf-8") as handle:
        for lineno, line in enumerate(handle, start=1):
            line = line.strip()
            if not line:
                continue
            try:
                entries.append(json.loads(line))
            except json.JSONDecodeError as exc:
                print(f"unparsable line {lineno}: {exc}")
    return entries


def expected_files(live_keys):
    expected = set()
    for key in live_keys:
        if "|" not in key:
            continue
        scene, rest = key.split("|", 1)
        if rest.startswith("clip:"):
            expected.add(scene + "__" + slug(rest[len("clip:"):]) + ".mp4")
        else:
            expected.add(scene + "__" + slug(rest) + ".png")
    return expected


def main(argv):
    if len(argv) not in (2, 3):
        print(f"usage: {argv[0]} <ui_preview_dir> [apply]")
        return 2
    out_dir = Path(argv[1])
    apply = len(argv) == 3 and argv[2] == "apply"
    if len(argv) == 3 and not apply:
        print(f"usage: {argv[0]} <ui_preview_dir> [apply]")
        return 2
    log_path = out_dir / "preview-log.jsonl"
    try:
        entries = load_entries(log_path)
    except FileNotFoundError:
        print(f"preview-clean: log not found: {log_path}")
        return 1
    except OSError as exc:
        print(f"preview-clean: cannot read {log_path}: {exc}")
        return 1
    if not entries:
        print("preview-clean: log is empty")
        return 1
    newest = entries[-1].get("run", "")
    live_keys = sorted(
        {
            entry.get("key", "")
            for entry in entries
            if entry.get("run") == newest
            and entry.get("key")
            and "|frame:" not in entry.get("key", "")
        }
    )
    live_scenes = sorted({key.split("|", 1)[0] for key in live_keys if "|" in key})
    expected = expected_files(live_keys)

    on_disk = sorted(
        path.name
        for path in out_dir.iterdir()
        if path.is_file() and path.suffix in (".png", ".mp4")
    )
    orphan_previews = [name for name in on_disk if name not in expected]

    cache_dir = out_dir / "_cache"
    sent_files = (
        sorted(path.name for path in cache_dir.iterdir() if path.name.endswith(".sent"))
        if cache_dir.is_dir()
        else []
    )
    orphan_sent = [
        name for name in sent_files if not (out_dir / name[: -len(".sent")]).exists()
    ]

    frames_dirs = sorted(
        path.name
        for path in out_dir.iterdir()
        if path.is_dir() and path.name.endswith("_frames")
    )
    live_set = set(live_scenes)
    stale_frames = [
        name for name in frames_dirs if name[: -len("_frames")] not in live_set
    ]

    print(f"preview-clean: dir={out_dir} newest run={newest}")
    print(f"live scenes ({len(live_scenes)}): {', '.join(live_scenes)}")
    print(f"live keys ({len(live_keys)}), expected files ({len(expected)})")
    print(f"orphan previews ({len(orphan_previews)}):")
    for name in orphan_previews:
        print(f"  {name}")
    print(f"orphan .sent ({len(orphan_sent)}):")
    for name in orphan_sent:
        print(f"  _cache/{name}")
    print(f"stale frames dirs ({len(stale_frames)}):")
    for name in stale_frames:
        print(f"  {name}/")

    if not apply:
        print("preview-clean: list only (pass `apply` to delete)")
        return 0

    for name in orphan_previews:
        (out_dir / name).unlink()
        print(f"deleted {name}")
    for name in orphan_sent:
        (cache_dir / name).unlink()
        print(f"deleted _cache/{name}")
    for name in stale_frames:
        shutil.rmtree(out_dir / name)
        print(f"deleted {name}/")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
