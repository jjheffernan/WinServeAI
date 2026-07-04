"""Agent backends: shell commands (stdin/stdout) or dry-run stubs."""

from __future__ import annotations

import json
import os
import subprocess
import textwrap
from pathlib import Path
from typing import Protocol

from .models import Issue, ReviewResult, ReviewStatus


class AgentError(RuntimeError):
    pass


class AgentBackend(Protocol):
    def code(self, task_prompt: str, worktree: Path, branch: str) -> str:
        """Apply feature work. Returns agent log/summary."""

    def review(self, diff: str, worktree: Path, branch: str) -> ReviewResult:
        """Review diff vs base. Must return structured ReviewResult."""

    def fix(self, issues: list[Issue], worktree: Path, branch: str) -> str:
        """Fix review issues. Returns agent log/summary."""


REVIEWER_INSTRUCTIONS = textwrap.dedent(
    """\
    You are the PR reviewer agent for WinServeAI.

    Review the git diff. Respond with ONLY valid JSON (no markdown fences):

    {
      "issues": [
        {
          "file": "path/to/file",
          "line": 12,
          "issue": "what is wrong",
          "severity": "error|warning|info",
          "suggestion": "how to fix"
        }
      ],
      "status": "PASS" | "FAIL"
    }

    Rules:
    - status PASS only when issues is empty
    - status FAIL when any issue has severity error (or any issue if unsure)
    - Never modify the base branch
    - Focus on correctness, architecture boundaries (UI must not own llama.cpp),
      and regressions
    """
)


CODER_INSTRUCTIONS = textwrap.dedent(
    """\
    You are the PR coder agent for WinServeAI.

    Work only inside the assigned worktree. Implement the task prompt.
    Respect architecture: UI → Server Manager → Backend trait → llama.
    Do not modify the base branch. Commit is handled by the orchestrator.
    """
)


FIXER_INSTRUCTIONS = textwrap.dedent(
    """\
    You are the PR fixer agent for WinServeAI.

    Fix the listed review issues only. Minimal diffs. Work only in the worktree.
    Commit is handled by the orchestrator.
    """
)


def _parse_review_json(raw: str) -> ReviewResult:
    text = raw.strip()
    if text.startswith("```"):
        # Tolerate fenced output from some agents.
        lines = text.splitlines()
        if lines and lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].strip() == "```":
            lines = lines[:-1]
        text = "\n".join(lines).strip()
    try:
        data = json.loads(text)
    except json.JSONDecodeError as exc:
        raise AgentError(f"reviewer did not return valid JSON: {exc}\n{raw[:500]}") from exc
    if not isinstance(data, dict):
        raise AgentError("reviewer JSON must be an object")
    return ReviewResult.from_dict(data)


def _run_cmd(cmd: str, stdin_payload: str, env: dict[str, str], cwd: Path) -> str:
    merged = os.environ.copy()
    merged.update(env)
    proc = subprocess.run(
        cmd,
        shell=True,
        input=stdin_payload,
        capture_output=True,
        text=True,
        cwd=str(cwd),
        env=merged,
    )
    if proc.returncode != 0:
        raise AgentError(
            f"agent command failed ({proc.returncode}): {proc.stderr.strip() or proc.stdout.strip()}"
        )
    return proc.stdout


class CommandAgent:
    """Invoke coder/reviewer/fixer via shell commands (agent-agnostic)."""

    def __init__(
        self,
        coder_cmd: str,
        reviewer_cmd: str,
        fixer_cmd: str,
    ) -> None:
        self.coder_cmd = coder_cmd
        self.reviewer_cmd = reviewer_cmd
        self.fixer_cmd = fixer_cmd

    def code(self, task_prompt: str, worktree: Path, branch: str) -> str:
        payload = json.dumps(
            {
                "instructions": CODER_INSTRUCTIONS,
                "task_prompt": task_prompt,
                "worktree": str(worktree),
                "branch": branch,
            }
        )
        return _run_cmd(
            self.coder_cmd,
            payload,
            {"PR_LOOP_ROLE": "coder", "PR_LOOP_BRANCH": branch, "PR_LOOP_WORKTREE": str(worktree)},
            worktree,
        )

    def review(self, diff: str, worktree: Path, branch: str) -> ReviewResult:
        payload = json.dumps(
            {
                "instructions": REVIEWER_INSTRUCTIONS,
                "branch": branch,
                "worktree": str(worktree),
                "diff": diff,
            }
        )
        raw = _run_cmd(
            self.reviewer_cmd,
            payload,
            {"PR_LOOP_ROLE": "reviewer", "PR_LOOP_BRANCH": branch, "PR_LOOP_WORKTREE": str(worktree)},
            worktree,
        )
        return _parse_review_json(raw)

    def fix(self, issues: list[Issue], worktree: Path, branch: str) -> str:
        payload = json.dumps(
            {
                "instructions": FIXER_INSTRUCTIONS,
                "branch": branch,
                "worktree": str(worktree),
                "issues": [i.to_dict() for i in issues],
            }
        )
        return _run_cmd(
            self.fixer_cmd,
            payload,
            {"PR_LOOP_ROLE": "fixer", "PR_LOOP_BRANCH": branch, "PR_LOOP_WORKTREE": str(worktree)},
            worktree,
        )


class DryRunAgent:
    """No-op agents for plumbing tests. Reviewer always PASSes."""

    def code(self, task_prompt: str, worktree: Path, branch: str) -> str:
        marker = worktree / ".pr_loop_dry_run.txt"
        marker.write_text(f"dry-run coder on {branch}\n{task_prompt}\n", encoding="utf-8")
        return f"dry-run coded {branch}"

    def review(self, diff: str, worktree: Path, branch: str) -> ReviewResult:
        return ReviewResult(issues=[], status=ReviewStatus.PASS)

    def fix(self, issues: list[Issue], worktree: Path, branch: str) -> str:
        return f"dry-run fix {len(issues)} issues on {branch}"


def build_agent(config_coder: str | None, config_reviewer: str | None, config_fixer: str | None, dry_run: bool) -> AgentBackend:
    if dry_run:
        return DryRunAgent()
    if not (config_coder and config_reviewer and config_fixer):
        raise AgentError(
            "coder_cmd, reviewer_cmd, and fixer_cmd are required unless dry_run is true"
        )
    return CommandAgent(config_coder, config_reviewer, config_fixer)
