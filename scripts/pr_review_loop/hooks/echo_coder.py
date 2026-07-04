#!/usr/bin/env python3
"""Example coder: records the task prompt; does not edit sources.

Wire a real agent here (Cursor Agent CLI, custom script, etc.).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> int:
    payload = json.load(sys.stdin)
    worktree = Path(payload["worktree"])
    branch = payload["branch"]
    note = worktree / ".pr_loop_coder_ran"
    note.write_text(
        f"branch={branch}\n{payload.get('task_prompt', '')}\n",
        encoding="utf-8",
    )
    print(f"coder stub ran for {branch}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
