#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "missing required file: $path" >&2
    exit 1
  fi
}

require_pattern() {
  local path="$1"
  local pattern="$2"
  local message="$3"
  if command -v rg >/dev/null 2>&1; then
    if rg -q "$pattern" "$path"; then
      return
    fi
  elif grep -Eq "$pattern" "$path"; then
    return
  fi
  if [[ -x "$(command -v rg 2>/dev/null)" || -x "$(command -v grep 2>/dev/null)" ]]; then
    echo "$message" >&2
    exit 1
  fi
  echo "missing required search tool: neither rg nor grep is available" >&2
  exit 1
}

require_file "rfcs/0004-cli-contract.md"
require_file "rfcs/0005-post-mvp-expansion.md"
require_file "crates/myx-core/src/lib.rs"
require_file ".github/pull_request_template.md"
require_file "CONTRIBUTING.md"
require_file "AGENTS.md"

require_pattern \
  "crates/myx-core/src/lib.rs" \
  'pub const SUPPORTED_TARGETS: &\[\&str\] = &\["openai", "mcp", "skill"\];' \
  "SUPPORTED_TARGETS must stay openai/mcp/skill for MVP"

require_pattern \
  "rfcs/0004-cli-contract.md" \
  'Command surface is limited to `init`, `add`, `inspect`, and `build`\.' \
  "RFC 0004 must define MVP command surface"

require_pattern \
  "rfcs/0004-cli-contract.md" \
  'Export targets are limited to Tier-1: `openai`, `mcp`, `skill`\.' \
  "RFC 0004 must define Tier-1 targets"

require_pattern \
  "rfcs/0004-cli-contract.md" \
  '`7`: Required semantic mismatch during export/build\.' \
  "RFC 0004 must document exit code 7"

require_pattern \
  ".github/pull_request_template.md" \
  '## MVP Contract Checklist' \
  "PR template must include MVP contract checklist"

require_pattern \
  "CONTRIBUTING.md" \
  '## MVP Contract Guardrails' \
  "CONTRIBUTING must include MVP guardrails section"

require_pattern \
  "AGENTS.md" \
  'Treat `rfcs/0004-cli-contract.md` as the MVP source of truth\.' \
  "AGENTS.md must require RFC 0004 as MVP authority"

echo "MVP contract checks passed."
