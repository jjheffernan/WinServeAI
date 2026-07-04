"""Shared types for the PR review loop."""

from __future__ import annotations

from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any


class ReviewStatus(str, Enum):
    PASS = "PASS"
    FAIL = "FAIL"


@dataclass
class Issue:
    file: str
    line: int | None
    issue: str
    severity: str
    suggestion: str

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> Issue:
        return cls(
            file=str(data.get("file", "")),
            line=data.get("line"),
            issue=str(data.get("issue", "")),
            severity=str(data.get("severity", "error")),
            suggestion=str(data.get("suggestion", "")),
        )

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


@dataclass
class ReviewResult:
    issues: list[Issue] = field(default_factory=list)
    status: ReviewStatus = ReviewStatus.PASS

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> ReviewResult:
        issues = [Issue.from_dict(i) for i in data.get("issues", [])]
        status_raw = str(data.get("status", "FAIL")).upper()
        status = ReviewStatus.PASS if status_raw == "PASS" else ReviewStatus.FAIL
        if issues and status == ReviewStatus.PASS:
            status = ReviewStatus.FAIL
        if not issues and status == ReviewStatus.FAIL:
            # Explicit FAIL with no issues still counts as FAIL.
            pass
        return cls(issues=issues, status=status)

    def to_dict(self) -> dict[str, Any]:
        return {
            "issues": [i.to_dict() for i in self.issues],
            "status": self.status.value,
        }


@dataclass
class BranchResult:
    branch: str
    status: ReviewStatus
    iterations: int
    remaining_issues: list[Issue] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "branch": self.branch,
            "status": self.status.value,
            "iterations": self.iterations,
            "remaining_issues": [i.to_dict() for i in self.remaining_issues],
        }


@dataclass
class LoopConfig:
    repo_path: str
    base_branch: str
    feature_branches: list[str]
    max_iterations: int = 5
    task_prompt: str = ""
    # Optional shell commands: stdin payload, stdout result.
    coder_cmd: str | None = None
    reviewer_cmd: str | None = None
    fixer_cmd: str | None = None
    # If True, skip agent cmds and use dry-run stubs (loop plumbing only).
    dry_run: bool = False
    # Worktrees live under <repo>/worktrees/wt-<branch>
    worktrees_dirname: str = "worktrees"
    push: bool = True

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> LoopConfig:
        branches = data.get("feature_branches") or data.get("feature_branches[]") or []
        if not isinstance(branches, list) or not branches:
            raise ValueError("feature_branches must be a non-empty list")
        max_iter = int(data.get("max_iterations", 5))
        if max_iter < 1:
            raise ValueError("max_iterations must be >= 1")
        return cls(
            repo_path=str(data["repo_path"]),
            base_branch=str(data.get("base_branch", "main")),
            feature_branches=[str(b) for b in branches],
            max_iterations=max_iter,
            task_prompt=str(data.get("task_prompt", "")),
            coder_cmd=data.get("coder_cmd"),
            reviewer_cmd=data.get("reviewer_cmd"),
            fixer_cmd=data.get("fixer_cmd"),
            dry_run=bool(data.get("dry_run", False)),
            worktrees_dirname=str(data.get("worktrees_dirname", "worktrees")),
            push=bool(data.get("push", True)),
        )
