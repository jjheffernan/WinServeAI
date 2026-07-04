"""Per-branch review loop and parallel runner."""

from __future__ import annotations

import json
import logging
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from .agents import AgentBackend, build_agent
from .gitops import commit_all, diff_vs_base, ensure_on_branch, push_feature
from .models import BranchResult, Issue, LoopConfig, ReviewStatus
from .worktrees import ensure_worktree

log = logging.getLogger("pr_review_loop")


def run_branch_loop(config: LoopConfig, branch: str, agent: AgentBackend) -> BranchResult:
    """
    Worktree loop for one feature branch.

    Constraints:
    - never modify base_branch
    - only operate inside assigned worktree
    - only push feature_branch
    - always re-diff before each review
    - stop on max_iterations even if FAIL
    """
    if branch == config.base_branch:
        raise ValueError(f"cannot run loop on base branch {config.base_branch!r}")

    repo = Path(config.repo_path).resolve()
    worktree = ensure_worktree(
        repo, branch, config.base_branch, config.worktrees_dirname
    )
    ensure_on_branch(worktree, branch, config.base_branch)

    log.info("[%s] worktree=%s", branch, worktree)

    # 1. Coder step
    agent.code(config.task_prompt, worktree, branch)
    if commit_all(worktree, "agent change"):
        log.info("[%s] committed agent change", branch)
    else:
        log.info("[%s] coder produced no changes", branch)
    if config.push:
        push_feature(worktree, branch, config.base_branch)

    iterations = 0
    last_result = None

    # 2–4. Review / fix loop
    while iterations < config.max_iterations:
        iterations += 1
        diff = diff_vs_base(worktree, config.base_branch)
        last_result = agent.review(diff, worktree, branch)
        log.info(
            "[%s] review iteration %s → %s (%s issues)",
            branch,
            iterations,
            last_result.status.value,
            len(last_result.issues),
        )

        if last_result.status == ReviewStatus.PASS:
            return BranchResult(
                branch=branch,
                status=ReviewStatus.PASS,
                iterations=iterations,
                remaining_issues=[],
            )

        agent.fix(last_result.issues, worktree, branch)
        if commit_all(worktree, "fix review issues"):
            log.info("[%s] committed review fixes", branch)
        if config.push:
            push_feature(worktree, branch, config.base_branch)

    remaining = last_result.issues if last_result else []
    return BranchResult(
        branch=branch,
        status=ReviewStatus.FAIL,
        iterations=iterations,
        remaining_issues=remaining,
    )


def run_all(config: LoopConfig) -> list[BranchResult]:
    """Run each feature branch loop in parallel."""
    agent = build_agent(
        config.coder_cmd, config.reviewer_cmd, config.fixer_cmd, config.dry_run
    )
    results: list[BranchResult] = []

    with ThreadPoolExecutor(max_workers=len(config.feature_branches)) as pool:
        futures = {
            pool.submit(run_branch_loop, config, branch, agent): branch
            for branch in config.feature_branches
        }
        for fut in as_completed(futures):
            branch = futures[fut]
            try:
                results.append(fut.result())
            except Exception as exc:  # noqa: BLE001 — surface per-branch failure
                log.exception("[%s] loop failed", branch)
                results.append(
                    BranchResult(
                        branch=branch,
                        status=ReviewStatus.FAIL,
                        iterations=0,
                        remaining_issues=[
                            Issue(
                                file="",
                                line=None,
                                issue=f"loop error: {exc}",
                                severity="error",
                                suggestion="Inspect logs; fix worktree/agent setup",
                            )
                        ],
                    )
                )

    # Stable order matching input branches.
    by_branch = {r.branch: r for r in results}
    return [by_branch[b] for b in config.feature_branches if b in by_branch]


def results_to_json(results: list[BranchResult]) -> str:
    return json.dumps([r.to_dict() for r in results], indent=2)
