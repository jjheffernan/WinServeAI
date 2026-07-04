#!/usr/bin/env python3
"""Enforce readiness score and technical-claim consistency (docs/policies/doc-drift.md)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
READINESS = ROOT / "docs" / "readiness"
DOCS = ROOT / "docs"

# Primary doc → expected module ids in banner (order flexible)
BANNER_DOCS: dict[str, list[str]] = {
    "backend.md": ["runtime-llama", "runtime-process", "server-manager"],
    "configuration.md": ["server-config"],
    "api.md": ["api", "server-health"],
    "logging.md": ["server-logs"],
    "development.md": ["cli", "scripts-ops"],
    "installer.md": ["installer", "bin"],
}

SCORECARD_RE = re.compile(
    r"\| Overall \| \*\*(?P<score>\d+\.\d+)\s*/\s*5\*\* \|\s*\n\| Label \| `(?P<label>[^`]+)` \|",
    re.MULTILINE,
)
DASHBOARD_RE = re.compile(
    r"\| (?P<id>[a-z0-9-]+) \| (?P<score>\d+\.\d+)/5 \| (?P<label>[a-z0-9-]+) \|",
)
BANNER_SCORE_RE = re.compile(
    r"(?P<id>[a-z0-9-]+)\s+(?P<score>\d+\.\d+)/5\s+\(`(?P<label>[^`]+)`\)"
    r"|(?P<score2>\d+\.\d+)/5\s+\(`(?P<label2>[^`]+)`\)"
)


def load_scorecards() -> dict[str, tuple[str, str]]:
    out: dict[str, tuple[str, str]] = {}
    for path in READINESS.glob("*.md"):
        if path.name in ("README.md", "PLAN.md"):
            continue
        text = path.read_text(encoding="utf-8")
        m = SCORECARD_RE.search(text)
        if not m:
            continue
        out[path.stem] = (m.group("score"), m.group("label"))
    return out


def load_dashboard() -> dict[str, tuple[str, str]]:
    text = (READINESS / "README.md").read_text(encoding="utf-8")
    out: dict[str, tuple[str, str]] = {}
    for m in DASHBOARD_RE.finditer(text):
        out[m.group("id")] = (m.group("score"), m.group("label"))
    return out


def parse_banner_scores(text: str, expected_ids: list[str]) -> list[tuple[str, str, str]]:
    """Return list of (id, score, label) from readiness banner line."""
    lines = [ln for ln in text.splitlines() if "Readiness:" in ln or "readiness:" in ln.lower()]
    if not lines:
        return []
    line = lines[0]
    found: list[tuple[str, str, str]] = []
    # Multi-module: "api 2.6/5 (`mvp-partial`), server-health 2.8/5 (`mvp-partial`)"
    for m in re.finditer(
        r"(?:(?P<id>[a-z0-9-]+)\s+)?(?P<score>\d+\.\d+)/5\s+\(`(?P<label>[^`]+)`\)",
        line,
    ):
        mid = m.group("id")
        score, label = m.group("score"), m.group("label")
        found.append((mid or "", score, label))
    # Assign ids if single score without id prefix
    if len(found) == 1 and not found[0][0] and len(expected_ids) == 1:
        found = [(expected_ids[0], found[0][1], found[0][2])]
    elif len(found) == len(expected_ids):
        # fill missing ids in order
        fixed = []
        for i, (mid, score, label) in enumerate(found):
            fixed.append((mid or expected_ids[i], score, label))
        found = fixed
    return found


def check_packages_layout(errors: list[str]) -> None:
    operator = [
        "api.md",
        "backend.md",
        "configuration.md",
        "contributing.md",
        "development.md",
        "installer.md",
        "logging.md",
        "release-process.md",
        "roadmap.md",
        "architecture.md",
        "vision.md",
    ]
    bad = re.compile(r"packages/(launcher|process|llama|api|hardware|config|logging|backend|shared)\b")
    for name in operator:
        path = DOCS / name
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        for i, line in enumerate(text.splitlines(), 1):
            if "Path note" in line or "do **not** reintroduce" in line.lower():
                continue
            if "no `packages/" in line.lower() or "not `packages/" in line.lower():
                continue
            if bad.search(line) and "→" not in line and "removed" not in line.lower():
                # allow explicit bans
                if "no backend" in line.lower() or "ban" in line.lower():
                    continue
                if "packages/" in line and ("must not" in line.lower() or "do not" in line.lower() or "never" in line.lower()):
                    continue
                # historical only in research — operator guides should not affirm packages as current
                if re.search(r"`packages/[a-z]+`", line) and "reintroduce" not in line.lower():
                    errors.append(f"{path.relative_to(ROOT)}:{i}: affirmative packages/* path")


def check_health_probe(errors: list[str]) -> None:
    health = (ROOT / "app/src/server/health.rs").read_text(encoding="utf-8")
    if "/v1/models" not in health:
        errors.append("app/src/server/health.rs: primary probe /v1/models missing")
    api = (DOCS / "api.md").read_text(encoding="utf-8")
    if "/v1/models" not in api:
        errors.append("docs/api.md: must document primary readiness GET /v1/models")


def check_maturity_headers(cards: dict[str, tuple[str, str]], errors: list[str]) -> None:
    dash = (READINESS / "README.md").read_text(encoding="utf-8")
    m = re.search(r"\*\*(\d+\.\d+)/5\*\*", dash)
    if not m:
        errors.append("docs/readiness/README.md: missing overall maturity **X.X/5**")
        return
    overall = m.group(1)
    for path in (DOCS / "TODO.md", DOCS / "PLAN.md"):
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        if overall not in text:
            errors.append(f"{path.relative_to(ROOT)}: maturity {overall} not reflected")


def main() -> int:
    errors: list[str] = []
    cards = load_scorecards()
    dash = load_dashboard()

    if not cards:
        errors.append("no readiness scorecards found")
    for mid, (score, label) in cards.items():
        if mid not in dash:
            errors.append(f"dashboard missing module {mid}")
            continue
        ds, dl = dash[mid]
        if ds != score or dl != label:
            errors.append(
                f"dashboard drift {mid}: card={score}/{label} dash={ds}/{dl}"
            )
    for mid in dash:
        if mid not in cards:
            errors.append(f"scorecard missing for dashboard module {mid}")

    for doc_name, ids in BANNER_DOCS.items():
        path = DOCS / doc_name
        if not path.exists():
            errors.append(f"missing primary doc {doc_name}")
            continue
        text = path.read_text(encoding="utf-8")
        if "Readiness:" not in text and "readiness dashboard" not in text.lower():
            errors.append(f"{doc_name}: missing Readiness banner")
            continue
        if doc_name == "architecture.md":
            continue
        found = parse_banner_scores(text, ids)
        if len(found) < len(ids):
            errors.append(f"{doc_name}: banner missing scores for {ids}")
            continue
        for mid, score, label in found:
            if mid not in cards:
                errors.append(f"{doc_name}: banner unknown module {mid}")
                continue
            cs, cl = cards[mid]
            if score != cs or label != cl:
                errors.append(
                    f"{doc_name}: banner drift {mid} banner={score}/{label} card={cs}/{cl}"
                )

    arch = DOCS / "architecture.md"
    if arch.exists() and "readiness/README.md" not in arch.read_text(encoding="utf-8"):
        errors.append("architecture.md: must link readiness dashboard")

    check_health_probe(errors)
    check_packages_layout(errors)
    check_maturity_headers(cards, errors)

    policy = DOCS / "policies" / "doc-drift.md"
    if not policy.exists():
        errors.append("missing docs/policies/doc-drift.md")

    plan = DOCS / "PLAN.md"
    if not plan.exists():
        errors.append("missing docs/PLAN.md")

    if errors:
        print("doc drift FAIL:")
        for e in errors:
            print(f"  - {e}")
        return 1
    print("doc drift OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
