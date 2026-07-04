"""CLI: python -m scripts.pr_review_loop --config examples/pr_review_loop.json"""

from __future__ import annotations

import argparse
import json
import logging
import sys
from pathlib import Path

from .loop import results_to_json, run_all
from .models import LoopConfig, ReviewStatus


def _load_config(path: Path) -> LoopConfig:
    data = json.loads(path.read_text(encoding="utf-8"))
    # repo_path is relative to cwd (repo root), not the config file.
    repo = Path(data.get("repo_path", ".")).expanduser()
    if not repo.is_absolute():
        repo = (Path.cwd() / repo).resolve()
    data["repo_path"] = str(repo)
    return LoopConfig.from_dict(data)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="PR worktree review loop — code, review, fix until PASS or max iterations"
    )
    parser.add_argument(
        "--config",
        "-c",
        type=Path,
        required=True,
        help="Path to JSON config (see examples/pr_review_loop.json)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Use stub agents (no external coder/reviewer/fixer)",
    )
    parser.add_argument(
        "--no-push",
        action="store_true",
        help="Skip git push (local commits only)",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Debug logging",
    )
    args = parser.parse_args(argv)

    logging.basicConfig(
        level=logging.DEBUG if args.verbose else logging.INFO,
        format="%(levelname)s %(message)s",
    )

    config = _load_config(args.config)
    if args.dry_run:
        config.dry_run = True
    if args.no_push:
        config.push = False

    results = run_all(config)
    print(results_to_json(results))

    if any(r.status != ReviewStatus.PASS for r in results):
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
