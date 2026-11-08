#!/usr/bin/env python3
"""Download SHA256-pinned upstream text notices; never execute downloaded content."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import urllib.request


def fetch(manifest_path: Path, destination: Path) -> dict:
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if destination.exists():
        raise ValueError("Destination must be a new directory.")
    planned = {}
    for entry in manifest["entries"]:
        for notice in entry["files"]:
            relative = Path(notice["path"])
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("Unsafe notice cache path.")
            if not re.fullmatch(r"https://raw\.githubusercontent\.com/[^/]+/[^/]+/[0-9a-f]{40}/[A-Za-z0-9_.-]+", notice["url"]):
                raise ValueError("Only immutable upstream GitHub text notices are accepted.")
            if not re.fullmatch(r"[0-9a-f]{64}", notice["sha256"]):
                raise ValueError("Each upstream notice requires a SHA256 pin.")
            previous = planned.setdefault(notice["path"], notice)
            if previous != notice:
                raise ValueError("Conflicting notice cache paths.")
    destination.mkdir(parents=True)
    for relative, notice in sorted(planned.items()):
        with urllib.request.urlopen(notice["url"], timeout=30) as response:
            data = response.read(256 * 1024 + 1)
        if len(data) > 256 * 1024 or hashlib.sha256(data).hexdigest() != notice["sha256"]:
            raise ValueError(f"Upstream notice failed size/hash verification: {relative}")
        output = destination / relative
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_bytes(data)
    (destination / "license-supplements.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return {"upstream_files": len(planned), "packages": len(manifest["entries"])}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(json.dumps(fetch(args.manifest, args.destination), indent=2))
    except (ValueError, OSError, KeyError) as error:
        parser.exit(1, f"Notice download failed: {error}\n")
