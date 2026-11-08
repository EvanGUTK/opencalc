#!/usr/bin/env python3
"""Collect unchanged upstream notices for the exact registry packages in Cargo.lock.

No Cargo invocation, network requests, source changes, or dependency execution.
Crates omitting license text require an explicitly SHA-pinned supplement manifest.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import tomllib
from urllib.parse import urlparse


NOTICE_NAME = re.compile(r"^(licen[cs]e|copying|copyright|notice|unlicense)([._-].*)?$", re.I)


def sha256(path: Path) -> str:
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def contained(path: Path, root: Path) -> Path:
    resolved = path.resolve(strict=True)
    if not resolved.is_relative_to(root.resolve(strict=True)) or path.is_symlink():
        raise ValueError(f"Notice path escapes its source directory: {path.name}")
    return resolved


def collect(lock_path: Path, vendor: Path, output: Path, supplements: Path | None) -> dict:
    if output.exists():
        raise ValueError("Output directory must not exist; use a new staging directory.")
    locked = tomllib.loads(lock_path.read_text(encoding="utf-8"))
    packages = sorted(
        (item for item in locked["package"] if item.get("source", "").startswith("registry+")),
        key=lambda item: (item["name"], item["version"]),
    )
    available = {}
    for manifest in sorted(vendor.glob("*/Cargo.toml")):
        metadata = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]
        available[(metadata["name"], metadata["version"])] = (manifest.parent, metadata)
    extra = {}
    excluded = {}
    if supplements:
        supplement_manifest = json.loads(supplements.read_text(encoding="utf-8"))
        for item in supplement_manifest["entries"]:
            key = (item["name"], item["version"])
            if key in extra:
                raise ValueError(f"Duplicate supplement: {key}")
            extra[key] = item["files"]
        for item in supplement_manifest.get("excluded_packages", []):
            key = (item["name"], item["version"])
            if key != ("dispatch", "0.2.0") or item.get("target") != "x86_64-pc-windows-msvc":
                raise ValueError("Unreviewed target-specific license exclusion.")
            excluded[key] = item
    plans = []
    missing = []
    for item in packages:
        key = (item["name"], item["version"])
        if key not in available:
            raise ValueError(f"Locked package missing from vendor: {key}")
        directory, metadata = available[key]
        files = []
        for candidate in sorted(directory.rglob("*")):
            if not candidate.is_file():
                continue
            relative = candidate.relative_to(directory)
            in_license_directory = any(part.lower() in ("licenses", "licences") for part in relative.parts[:-1])
            declared_license_file = metadata.get("license-file") == relative.as_posix()
            if NOTICE_NAME.match(candidate.name) or in_license_directory or declared_license_file:
                source = contained(candidate, directory)
                files.append((source, relative.as_posix(), None))
        for number, supplement in enumerate(extra.get(key, [])):
            source = contained(supplements.parent / supplement["path"], supplements.parent)
            expected = supplement["sha256"].lower()
            if not re.fullmatch(r"[0-9a-f]{64}", expected) or sha256(source) != expected:
                raise ValueError(f"Supplement hash mismatch: {key}")
            url = supplement["url"]
            if not url.startswith("https://"):
                raise ValueError(f"Supplement requires HTTPS provenance: {key}")
            filename = Path(urlparse(url).path).name
            if not re.fullmatch(r"[A-Za-z0-9_.-]+", filename):
                raise ValueError(f"Unsafe upstream notice filename: {key}")
            files.append((source, f"upstream/{number:02d}-{filename}", url))
        if not files and key not in excluded:
            missing.append(f"{key[0]} {key[1]} ({metadata.get('license', 'unspecified')})")
        plans.append((item, metadata, files))
    if missing:
        raise ValueError("Missing upstream license notices:\n" + "\n".join(missing))
    output.mkdir(parents=True)
    index = {"format": 1, "cargo_lock_sha256": sha256(lock_path), "scope": "Locked registry-package notice superset for Windows; includes build and other-platform notices. Any package without notice files records an explicit, reviewed Windows-target exclusion.", "packages": []}
    for item, metadata, files in plans:
        package = {"name": item["name"], "version": item["version"], "license": metadata.get("license"), "repository": metadata.get("repository"), "files": []}
        if (item["name"], item["version"]) in excluded:
            package["excluded_from_binary"] = excluded[(item["name"], item["version"])]
        for source, original, url in files:
            relative = Path("packages") / f"{item['name']}-{item['version']}" / original
            destination = output / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, destination)
            notice = {"path": relative.as_posix(), "sha256": sha256(destination), "upstream_path": original}
            if url:
                notice["source_url"] = url
            package["files"].append(notice)
        index["packages"].append(package)
    (output / "index.json").write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (output / "README.txt").write_text("OpenCalc dependency notices\n\nThese unchanged upstream license and notice texts accompany the locked Windows\nbuild. The package index records declared license expressions, repositories,\nnotice hashes, and immutable supplement URLs where published archives omitted\nlicense texts. The notice superset includes other-platform dependencies; any\npackage without notice files records an explicit reviewed Windows-target\nexclusion. OpenCalc's MIT license does not replace these terms. Font notices\nare also in assets/licenses.\n", encoding="utf-8")
    return {"packages": len(index["packages"]), "notices": sum(len(item["files"]) for item in index["packages"]), "index_sha256": sha256(output / "index.json")}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lock", type=Path, required=True)
    parser.add_argument("--vendor", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--supplements", type=Path)
    args = parser.parse_args()
    try:
        print(json.dumps(collect(args.lock, args.vendor, args.output, args.supplements), indent=2))
    except (ValueError, KeyError, OSError) as error:
        parser.exit(1, f"License collection failed: {error}\n")


if __name__ == "__main__":
    main()
