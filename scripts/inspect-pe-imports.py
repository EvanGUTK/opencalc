#!/usr/bin/env python3
"""Read PE import tables without running the executable or installing packages."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import struct


def inspect(path: Path) -> dict:
    data = path.read_bytes()
    def integer(offset: int, kind: str = "I") -> int:
        if offset < 0 or offset + struct.calcsize("<" + kind) > len(data):
            raise ValueError("Truncated PE structure.")
        return struct.unpack_from("<" + kind, data, offset)[0]
    if data[:2] != b"MZ":
        raise ValueError("Missing MZ signature.")
    pe = integer(0x3C)
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Missing PE signature.")
    machine = integer(pe + 4, "H")
    section_count = integer(pe + 6, "H")
    optional_size = integer(pe + 20, "H")
    optional = pe + 24
    if machine != 0x8664 or integer(optional, "H") != 0x20B:
        raise ValueError("Expected Windows x86_64 PE32+ image.")
    image_base = integer(optional + 24, "Q")
    sections = []
    for number in range(section_count):
        offset = optional + optional_size + number * 40
        sections.append((integer(offset + 12), max(integer(offset + 8), integer(offset + 16)), integer(offset + 20)))
    def file_offset(rva: int) -> int:
        for start, length, raw in sections:
            if start <= rva < start + length:
                result = raw + rva - start
                if result >= len(data):
                    raise ValueError("Import RVA exceeds image bytes.")
                return result
        raise ValueError("Import RVA lies outside mapped sections.")
    def name(rva: int) -> str:
        offset = file_offset(rva)
        end = data.find(b"\0", offset, min(offset + 1024, len(data)))
        if end < 0:
            raise ValueError("Unterminated imported DLL name.")
        return data[offset:end].decode("ascii")
    imports = set()
    # PE32+ data directories: import index1 and delay-import index13.
    for directory_index, descriptor_size in ((1, 20), (13, 32)):
        directory = optional + 112 + directory_index * 8
        if directory + 8 > optional + optional_size:
            continue
        rva, size = integer(directory), integer(directory + 4)
        if not rva or not size:
            continue
        offset = file_offset(rva)
        for cursor in range(offset, min(offset + size, len(data)) - descriptor_size + 1, descriptor_size):
            if not any(data[cursor:cursor + descriptor_size]):
                break
            if directory_index == 1:
                imports.add(name(integer(cursor + 12)))
            else:
                attributes, name_address = integer(cursor), integer(cursor + 4)
                imports.add(name(name_address if attributes & 1 else name_address - image_base))
    redistributable = sorted(dll for dll in imports if re.match(r"(?i)^(vcruntime|msvcp|concrt|msvcr)\d", dll))
    return {"architecture": "x86_64", "sha256": hashlib.sha256(data).hexdigest(), "imports": sorted(imports, key=str.casefold), "visual_cpp_redistributable_imports": redistributable, "separate_vc_runtime_required": bool(redistributable)}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("executable", type=Path)
    parser.add_argument("--require-static-crt", action="store_true")
    args = parser.parse_args()
    try:
        report = inspect(args.executable)
        print(json.dumps(report, indent=2))
        if args.require_static_crt and report["separate_vc_runtime_required"]:
            parser.exit(1, "Executable imports a separate Visual C++ redistributable runtime.\n")
    except (OSError, ValueError, struct.error) as error:
        parser.exit(1, f"PE inspection failed: {error}\n")
