#!/usr/bin/env bash
set -euo pipefail

OWNER="myx-hq"
REPO="myx-hq/myx"
PROJECT_TITLE="MVP Hardening Audit"

require_gh_auth() {
  if ! gh auth status >/dev/null 2>&1; then
    echo "GitHub CLI is not authenticated." >&2
    echo "Run: gh auth login -h github.com" >&2
    echo "If needed for project operations: gh auth refresh -s project" >&2
    exit 1
  fi
}

ensure_project() {
  if gh project list --owner "$OWNER" --limit 200 | rg -F "$PROJECT_TITLE" >/dev/null; then
    echo "Project '$PROJECT_TITLE' already exists for owner '$OWNER'."
    return
  fi

  echo "Creating project '$PROJECT_TITLE'..."
  gh project create --owner "$OWNER" --title "$PROJECT_TITLE" >/dev/null
}

create_issue() {
  local title="$1"
  local body="$2"

  echo "Creating issue: $title"
  printf "%s" "$body" | gh issue create \
    --repo "$REPO" \
    --title "$title" \
    --body-file - \
    --project "$PROJECT_TITLE" \
    >/dev/null
}

require_gh_auth
ensure_project

create_issue \
  "P0: Verify index digest across full package payload" \
  "## Problem
Index digest validation currently hashes profile content only, which allows non-profile package drift to pass integrity checks.

## Scope
- Define deterministic package digest algorithm (full package payload, path-stable ordering).
- Validate against index digest during \`myx add\`.
- Add tests for tampered non-profile files.

## Acceptance Criteria
- Digest covers all package files included in install payload.
- Integrity check fails on any payload mutation.
- RFC/docs/schema references are updated in same PR."

create_issue \
  "P0: Make store install atomic and non-destructive" \
  "## Problem
Store install currently deletes existing package path before copy and is not atomic, risking partial installs.

## Scope
- Introduce staged copy into temp dir.
- Atomic rename/swap to target store path.
- Rollback-safe failure semantics.

## Acceptance Criteria
- No destructive pre-delete of active store path.
- Failed copy leaves previous package intact.
- Tests cover interrupted/failed installs and final state correctness."

create_issue \
  "P1: Fix MCP wrapper subprocess cwd semantics" \
  "## Problem
MCP runtime config execution currently resolves relative cwd from wrapper artifact location, not package/workspace intent.

## Scope
- Define explicit runtime base directory contract.
- Add package/workspace root fields into MCP runtime config.
- Enforce cwd resolution consistently in executor and wrapper.

## Acceptance Criteria
- Relative cwd behavior is deterministic and documented.
- Subprocess tools run from expected base dir in MCP mode.
- Tests cover cwd allowlist checks and execution behavior."

create_issue \
  "P1: Align loss-report schema with emitted payload" \
  "## Problem
Implementation emits structured loss fields that are not represented in the loss-report schema.

## Scope
- Update \`schemas/loss-report/v1/schema.json\` to match emitted report shape.
- Ensure RFC/docs describe fields and required semantics.
- Add schema validation tests for emitted reports.

## Acceptance Criteria
- Emitted loss reports validate against schema.
- Required mismatch semantics are represented in schema.
- Contract docs and implementation are aligned."

create_issue \
  "P1: Strengthen capability profile schema to match runtime validation" \
  "## Problem
Profile schema currently under-specifies execution and permission constraints compared to runtime validator behavior.

## Scope
- Tighten schema requirements for execution blocks.
- Capture subprocess constraints needed by MVP enforcement.
- Align schema, validator, and docs.

## Acceptance Criteria
- Schema-valid profiles do not fail runtime validation for missing required MVP fields.
- Schema and runtime checks are consistent for HTTP/subprocess declarations.
- Tests cover both valid and invalid profile fixtures."

create_issue \
  "P2: Add strict MCP protocol compatibility mode" \
  "## Problem
\`myx-mcp-wrapper\` currently uses a simplified line-delimited JSON loop and is not a full MCP transport implementation.

## Scope
- Add strict MCP-compatible protocol mode.
- Preserve current simple mode if needed for local testing.
- Document interop behavior and constraints.

## Acceptance Criteria
- Wrapper supports MCP-compatible request/response framing.
- Interop smoke test exists for supported MCP clients or fixtures.
- Mode selection and defaults are documented."

create_issue \
  "P2: Improve non-interactive policy behavior defaults" \
  "## Problem
Non-interactive behavior relies heavily on explicit flag usage and can be made safer/more predictable.

## Scope
- Define deterministic non-interactive detection strategy (flags/env/TTY behavior).
- Align policy decisions and messaging for CI workflows.
- Update RFC/docs and tests.

## Acceptance Criteria
- Non-interactive installs have explicit, deterministic behavior without implicit prompts.
- Behavior is fully documented and test-covered.
- No regression for interactive review-required mode."

create_issue \
  "P2: Refactor myx-cli into module-based command handlers" \
  "## Problem
\`myx-cli/src/main.rs\` is monolithic and mixes command handling, export logic, and tests.

## Scope
- Split commands into modules (\`init/add/inspect/build\`).
- Isolate output contract helpers and build target emitters.
- Keep behavior unchanged while improving maintainability.

## Acceptance Criteria
- Functional behavior remains unchanged.
- Module boundaries are clear and test coverage preserved or improved.
- Clippy/tests remain clean."

create_issue \
  "P2: Align adapter/docs architecture with MVP reality" \
  "## Problem
RFC 0003 and high-level overview text still describe pre-MVP adapter/plugin assumptions that conflict with current built-in Rust MVP architecture.

## Scope
- Rewrite RFC 0003 to reflect MVP built-in adapter contract.
- Update overview docs to avoid implying shipped registry/publish workflows in MVP.
- Ensure RFC 0004/0005 references remain consistent.

## Acceptance Criteria
- Architecture docs are internally consistent.
- MVP vs post-MVP boundaries are clear.
- No conflicting adapter model language remains."

echo "Project and issues creation complete."
