#!/usr/bin/env python3
"""Extract a CHANGELOG.md section for a release tag.

Usage:
  python3 scripts/changelog_notes.py v0.1.0
  python3 scripts/changelog_notes.py v0.1.0-rc.1

Prints the matching ## [version] body, or [Unreleased] for preview tags
when no exact heading exists. Exits 1 if CHANGELOG.md is missing/empty.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CHANGELOG = ROOT / "CHANGELOG.md"


def version_from_tag(tag: str) -> str:
    t = tag.strip()
    if t.startswith("v") or t.startswith("V"):
        t = t[1:]
    return t


def sections(text: str) -> list[tuple[str, str]]:
    """Return [(heading_version_or_Unreleased, body), ...] for ## [..] headings."""
    parts = re.split(r"(?m)^##\s+\[([^\]]+)\][^\n]*\n", text)
    # parts[0] is preamble; then pairs of (name, body)
    out: list[tuple[str, str]] = []
    i = 1
    while i + 1 < len(parts):
        out.append((parts[i].strip(), parts[i + 1].strip()))
        i += 2
    return out


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: changelog_notes.py <tag>", file=sys.stderr)
        return 2
    tag = sys.argv[1]
    ver = version_from_tag(tag)
    if not CHANGELOG.is_file():
        print("CHANGELOG.md missing", file=sys.stderr)
        return 1
    text = CHANGELOG.read_text(encoding="utf-8")
    secs = sections(text)
    if not secs:
        print("CHANGELOG.md has no ## [version] sections", file=sys.stderr)
        return 1

    body = None
    for name, content in secs:
        if name == ver:
            body = content
            break
    if body is None and "-rc." in ver:
        for name, content in secs:
            if name.lower() == "unreleased":
                body = content
                break
    if body is None:
        print(f"no CHANGELOG section for {ver!r} (and no Unreleased fallback)", file=sys.stderr)
        return 1

    print(f"# WinServeAI {tag}\n")
    print(body.rstrip())
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
