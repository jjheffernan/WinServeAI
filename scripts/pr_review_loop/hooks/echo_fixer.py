#!/usr/bin/env python3
"""Example fixer: logs issues; does not edit sources."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    payload = json.load(sys.stdin)
    worktree = Path(payload["worktree"])
    issues = payload.get("issues", [])
    note = worktree / ".pr_loop_fixer_ran"
    note.write_text(json.dumps(issues, indent=2), encoding="utf-8")
    print(f"fixer stub saw {len(issues)} issues")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
