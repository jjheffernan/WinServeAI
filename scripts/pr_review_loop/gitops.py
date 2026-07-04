"""Git operations constrained to a single worktree and feature branch."""

from __future__ import annotations

import subprocess
from pathlib import Path


class GitError(RuntimeError):
    pass


def run_git(worktree: Path, *args: str, check: bool = True) -> str:
    cmd = ["git", "-C", str(worktree), *args]
    proc = subprocess.run(cmd, capture_output=True, text=True)
    if check and proc.returncode != 0:
        raise GitError(
            f"git {' '.join(args)} failed ({proc.returncode}): {proc.stderr.strip()}"
        )
    return proc.stdout


def current_branch(worktree: Path) -> str:
    return run_git(worktree, "rev-parse", "--abbrev-ref", "HEAD").strip()


def assert_feature_branch(worktree: Path, feature_branch: str, base_branch: str) -> None:
    if feature_branch == base_branch:
        raise GitError(f"refusing to operate on base branch {base_branch!r}")
    head = current_branch(worktree)
    if head not in (feature_branch, "HEAD"):
        # Detached HEAD is ok if it points at the feature branch tip.
        tip = run_git(worktree, "rev-parse", "HEAD").strip()
        branch_tip = run_git(worktree, "rev-parse", feature_branch).strip()
        if tip != branch_tip:
            raise GitError(
                f"worktree HEAD is {head!r}, expected feature branch {feature_branch!r}"
            )


def ensure_on_branch(worktree: Path, feature_branch: str, base_branch: str) -> None:
    if feature_branch == base_branch:
        raise GitError(f"refusing to checkout base branch {base_branch!r}")
    run_git(worktree, "checkout", feature_branch)


def commit_all(worktree: Path, message: str) -> bool:
    """Stage all and commit. Returns False if there was nothing to commit."""
    run_git(worktree, "add", "-A")
    staged = run_git(worktree, "diff", "--cached", "--name-only").strip()
    if not staged:
        return False
    run_git(worktree, "commit", "-m", message)
    return True


def push_feature(worktree: Path, feature_branch: str, base_branch: str) -> None:
    if feature_branch == base_branch:
        raise GitError(f"refusing to push base branch {base_branch!r}")
    assert_feature_branch(worktree, feature_branch, base_branch)
    run_git(worktree, "push", "-u", "origin", feature_branch)


def diff_vs_base(worktree: Path, base_branch: str) -> str:
    """Fresh diff vs origin/<base> (or local base if origin missing)."""
    remote_base = f"origin/{base_branch}"
    # Prefer origin/base; fall back to local base.
    has_remote = (
        subprocess.run(
            ["git", "-C", str(worktree), "rev-parse", "--verify", remote_base],
            capture_output=True,
        ).returncode
        == 0
    )
    base_ref = remote_base if has_remote else base_branch
    return run_git(worktree, "diff", f"{base_ref}...HEAD")
