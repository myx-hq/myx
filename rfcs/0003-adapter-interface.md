# RFC 0003: Built-In Adapter Contract (MVP)

*Status: Accepted*

## Summary

This RFC defines the adapter contract for myx MVP (v0) as implemented by the Rust core.

For MVP:

- Export adapters are built in and selected by exact target id: `openai`, `mcp`, `skill`.
- Adapter behavior is deterministic and produces explicit loss reports.
- Dynamic adapter discovery and external plugin loading are out of scope.

This RFC aligns with RFC 0004 (MVP CLI contract) and RFC 0005 (post-MVP expansion).

## Motivation

Earlier drafts described a generic third-party adapter/plugin interface. That model does not match MVP implementation scope.

MVP needs a tighter contract:

1. one deterministic build path,
2. one enforcement model,
3. one target set.

The goal is to prove install/inspect/build/runtime correctness before opening the adapter surface.

## MVP Scope

### In Scope

- Built-in export target handlers for `openai`, `mcp`, `skill`.
- Deterministic artifact emission under `.myx/<target>/...`.
- Structured loss-report generation and required-mismatch hard failures.
- Validation coupling to Capability Profile v1 (`tool_class`, `execution`, permissions).

### Out of Scope (Deferred to RFC 0005)

- External adapter plugins.
- Runtime adapter discovery/registration commands (including `myx list-adapters`).
- Adapter interfaces intended for arbitrary third-party runtime loading.

## Adapter Contract (Normative)

### Inputs

Each built-in export receives:

1. a fully validated package profile,
2. a destination output directory,
3. target-specific context (for example package base directory for MCP runtime config).

Validation must complete before adapter execution begins.

### Outputs

Each export returns:

1. runtime artifacts written to output directory,
2. zero or more structured issues describing lossy or incompatible semantics.

Issue fields:

- `level`
- `category`
- `tool` (optional)
- `message`
- `required_mismatch` (boolean)

If any issue has `required_mismatch = true`, build must fail (RFC 0004 exit code `7`) after writing `loss-report.json`.

## Built-In Target Requirements

### `openai`

Must emit deterministic:

- `tools.json`
- `instructions.md`

If a tool requires subprocess execution semantics that cannot be preserved, emit required mismatch.

### `skill`

Must emit deterministic:

- `SKILL.md`

If tool semantics are not runnable/preservable in documentation-only output, emit required mismatch.

### `mcp`

Must emit deterministic:

- `server.json`
- `runtime-config.json`
- `launch.json`
- `run.sh`

Generated launch behavior must use strict MCP framing mode (`--protocol mcp`) and route execution through the global runtime executor.

## Import Adapters in MVP

Importers are non-blocking for MVP ship. Existing importer crates may evolve independently, but importer completeness is not a release gate for v0.

## Determinism and Safety Requirements

All built-in adapters must:

1. avoid mutating source package files,
2. produce byte-stable output for unchanged inputs,
3. surface semantic loss explicitly (no silent drops),
4. preserve policy/runtime enforcement boundaries (no direct shell shortcuts outside executor rules).

## Post-MVP Extension Boundary

External adapter/plugin design is intentionally deferred. Any future plugin architecture must preserve the same loss-report and policy guarantees defined in MVP RFCs.
