#!/usr/bin/env bash
# Restore agent-agnostic skills into .agents/skills/ from skills-lock.json.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

npx skills experimental_install
echo "Skills restored under .agents/skills/"
