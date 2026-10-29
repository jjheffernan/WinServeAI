#!/usr/bin/env python3
"""Check that release version files agree, and that a tag matches them.

No argument: the version strings must be identical and bin/VERSION must be a
llama.cpp b#### pin.

One argument (a tag): it must be v{version} or v{version}-rc.N for that version.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def cargo_version(path: Path) -> str:
    match = re.search(r'version\s*=\s*"([^"]+)"', path.read_text(encoding="utf-8"))
    if not match:
        raise SystemExit(f"{path.relative_to(ROOT)}: no version")
    return match.group(1)


def versions() -> dict[str, str]:
    iss = (ROOT / "installer/inno/WinServeAI.iss").read_text(encoding="utf-8")
    iss_match = re.search(r'MyAppVersion "([^"]+)"', iss)
    if not iss_match:
        raise SystemExit("installer/inno/WinServeAI.iss: no MyAppVersion")
    tray_conf = json.loads(
        (ROOT / "clients/tray/src-tauri/tauri.conf.json").read_text(encoding="utf-8")
    )
    return {
        "Cargo.toml": cargo_version(ROOT / "Cargo.toml"),
        "server/Cargo.toml": cargo_version(ROOT / "server/Cargo.toml"),
        "clients/tray/src-tauri/Cargo.toml": cargo_version(
            ROOT / "clients/tray/src-tauri/Cargo.toml"
        ),
        "clients/tray/src-tauri/tauri.conf.json": str(tray_conf.get("version", "")),
        "installer/inno/WinServeAI.iss": iss_match.group(1),
    }


def pin() -> str:
    text = (ROOT / "bin/VERSION").read_text(encoding="utf-8").strip()
    if not re.fullmatch(r"b\d+", text):
        raise SystemExit(f"bin/VERSION: expected b####, got {text!r}")
    return text


def check_tag(tag: str, version: str) -> None:
    stable = re.fullmatch(r"v(\d+\.\d+\.\d+)", tag)
    preview = re.fullmatch(r"v(\d+\.\d+\.\d+)-rc\.([1-9]\d*)", tag)
    found = stable or preview
    if not found:
        raise SystemExit(
            f"{tag}: use v{version} on main, or v{version}-rc.N on dev"
        )
    if found.group(1) != version:
        raise SystemExit(f"{tag}: version {found.group(1)} != package {version}")


def main() -> int:
    found = versions()
    unique = set(found.values())
    if len(unique) != 1 or "" in unique:
        lines = "\n".join(f"  {name}: {value}" for name, value in found.items())
        raise SystemExit(f"version files disagree:\n{lines}")
    version = unique.pop()
    llama = pin()
    if len(sys.argv) > 2:
        raise SystemExit("usage: check_release_tag.py [vX.Y.Z | vX.Y.Z-rc.N]")
    if len(sys.argv) == 2:
        check_tag(sys.argv[1], version)
    print(f"OK: {version} llama.cpp {llama}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
