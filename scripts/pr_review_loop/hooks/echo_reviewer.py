#!/usr/bin/env python3
"""Example reviewer: reads JSON payload on stdin, always PASSes.

Replace with a real agent CLI that prints the required review JSON.
"""

from __future__ import annotations

import json
import sys


def main() -> int:
    payload = json.load(sys.stdin)
    _ = payload.get("diff", "")
    json.dump({"issues": [], "status": "PASS"}, sys.stdout)
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
