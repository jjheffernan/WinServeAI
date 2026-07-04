"""Create and resolve per-feature worktrees under repo/worktrees/."""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

from .gitops import GitError


def sanitize_branch(branch: str) -> str:
    """Map feature/foo -> feature-foo for directory names."""
    name = branch.strip().replace("/", "-").replace("\\", "-")
    name = re.sub(r"[^A-Za-z0-9._-]+", "-", name)
    name = name.strip(".-") or "branch"
    return name


def worktree_path(repo: Path, branch: str, worktrees_dirname: str = "worktrees") -> Path:
    return repo / worktrees_dirname / f"wt-{sanitize_branch(branch)}"


def ensure_worktree(
    repo: Path,
    branch: str,
    base_branch: str,
    worktrees_dirname: str = "worktrees",
) -> Path:
    """
    Add worktree at repo/worktrees/wt-<branch> for feature_branch.

    Never checks out or modifies base_branch.
    """
    if branch == base_branch:
        raise GitError(f"refusing to create worktree for base branch {base_branch!r}")

    repo = repo.resolve()
    path = worktree_path(repo, branch, worktrees_dirname)
    path.parent.mkdir(parents=True, exist_ok=True)

    if path.exists():
        # Reuse existing worktree if it is already registered.
        return path

    # Ensure branch exists (create from base if missing — does not modify base tip).
    has_branch = (
        subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--verify", branch],
            capture_output=True,
        ).returncode
        == 0
    )
    if not has_branch:
        proc = subprocess.run(
            ["git", "-C", str(repo), "branch", branch, base_branch],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            raise GitError(f"failed to create branch {branch!r}: {proc.stderr.strip()}")

    proc = subprocess.run(
        ["git", "-C", str(repo), "worktree", "add", str(path), branch],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        raise GitError(f"worktree add failed for {branch!r}: {proc.stderr.strip()}")

    return path
