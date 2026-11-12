#!/usr/bin/env python3
"""Find where a piece of code sits in the reference Java client, for MECHANICS.md citations.

Usage:
    python scripts/cite.py <reference-root> <File.java> "<exact text>"

Prints every match as `jagex/File.java:LINE @ 0xOFFSET` plus the line itself. The reference is
read as Latin-1 (its sources are not valid UTF-8), so offsets are byte offsets in the file.
Exit status 1 when the text isn't found: don't cite what you couldn't find.
"""
import pathlib
import sys


def main() -> int:
    if len(sys.argv) != 4:
        print(__doc__)
        return 2
    root, name, text = pathlib.Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    matches = list(root.rglob(name))
    if not matches:
        print(f"no file named {name} under {root}")
        return 1
    found = 0
    needle = text.encode("latin-1")
    for path in matches:
        data = path.read_bytes()
        at = data.find(needle)
        while at != -1:
            line = data.count(b"\n", 0, at) + 1
            start = data.rfind(b"\n", 0, at) + 1
            end = data.find(b"\n", at)
            shown = data[start:end if end != -1 else len(data)].decode("latin-1").strip()
            rel = path.relative_to(root).as_posix()
            short = rel.split("com/", 1)[-1]
            print(f"{short}:{line} @ {hex(at)}  |  {shown}")
            found += 1
            at = data.find(needle, at + 1)
    if not found:
        print(f"'{text}' not found in {name}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
