# RFC 0004: MVP CLI Contract (Rust Core)

*Status: Draft*

## Summary

This RFC defines the normative CLI and runtime contract for `myx` MVP (v0):

- Rust core implementation, distributed as a binary (macOS-first via Homebrew).
- Command surface is limited to `init`, `add`, `inspect`, and `build`.
- Export targets are limited to Tier-1: `openai`, `mcp`, `skill`.
- Package discovery uses local paths and project-configured static indexes.
- Policy enforcement is mandatory and deterministic.

Anything removed from prior broader CLI scope is deferred to RFC 0005.

## Motivation

MVP must prove one loop with high confidence:

1. Install package.
2. Inspect metadata and permissions.
3. Build deterministic target artifacts.
4. Run target outputs with explicit security boundaries.

Scope beyond this loop increases risk without improving the core proof.

## MVP Scope

### In Scope

- Commands: `myx init`, `myx add`, `myx inspect`, `myx build`.
- Targets: `openai`, `mcp`, `skill`.
- Sources: local package path and static index package resolution.
- Runtime execution model: global runtime executor with declarative `http` and `subprocess` tool actions.
- Lockfile: deterministic, atomic writes on successful install only.

### Out of Scope

Deferred to RFC 0005:

- `myx publish`
- `myx list-adapters`
- Hosted registry API and auth flows
- Additional targets (`vercel`, `gemini`, `claude`)
- External adapter plugin system

## Command Contracts

### `myx init`

Purpose:
- Scaffold a package directory.

Behavior:
1. Create baseline files when target is empty or missing:
- `myx.yaml`
- `capability.json`
- `prompts/system.md`
- `tools/` (empty or sample schema)
2. Fail if required files exist unless `--force` is set.
3. Ensure scaffold metadata and IR pointers are internally consistent.

### `myx add <name|path>`

Purpose:
- Install a package into the local workspace/store and update `myx.lock`.

Behavior:
1. Resolve source:
- If `<name|path>` is an existing path, treat as local package.
- Otherwise resolve by package name using configured static indexes.
2. Validate package structure, checksums, and profile compatibility.
3. Evaluate permissions against policy.
4. Install to local store/workspace state.
5. Atomically update `myx.lock`.

Failure rules:
- Any validation, resolution, integrity, or policy failure leaves `myx.lock` unchanged.
- Partial install state must not be considered active.

### `myx inspect <name|path>`

Purpose:
- Display package identity, capabilities, tool classes, execution declarations, and permissions.

Behavior:
- Human-readable output by default.
- `--json` emits stable machine-readable output.
- Missing package or invalid source returns non-zero exit code.

### `myx build --target <openai|mcp|skill>`

Purpose:
- Export deterministic runtime artifacts from package IR/profile.

Behavior:
1. Load and validate package profile.
2. Resolve built-in adapter by exact target id.
3. Generate artifacts under `.myx/<target>/...`.
4. Emit loss report when lossy conversion occurs.
5. Hard-fail on required semantic mismatch.

Rules:
- Repeated builds with unchanged inputs must produce byte-stable outputs.
- Export must not mutate package source files.

MCP wrapper protocol modes:
- Wrapper must support strict MCP framing mode (`--protocol mcp`) using `Content-Length` framed JSON-RPC messages over stdio.
- Wrapper may also support a simplified line-delimited mode (`--protocol simple`) for local debugging.
- Generated MCP launch artifacts must invoke strict MCP mode explicitly.

## Capability Profile v1 Requirements

For MVP-targetable packages, each tool must include:

- Required `tool_class` enum:
  - `http_api`
  - `local_process`
  - `filesystem_assisted`
  - `composite`
- Required `execution` block with `kind`:
  - `http`
  - `subprocess`

Missing required profile fields are validation errors.

## Policy and Security Contract

Default policy mode is review-required.

### Interactive

- User must explicitly approve requested permissions before install completes.

### Non-interactive / CI

- Install is denied unless permissions are pre-approved by explicit allowlist configuration.
- Non-interactive mode detection is deterministic with this precedence:
  1. `--non-interactive` flag.
  2. `MYX_NON_INTERACTIVE` env override (`1/0`, `true/false`, `yes/no`, `on/off`).
  3. Truthy `CI` env.
  4. Non-TTY stdio (`stdin` or `stdout` is not a terminal).
- In non-interactive mode, prompts are never shown.

### Subprocess Constraints (MVP)

For `execution.kind = subprocess`, executor enforcement must include:

- Exact command allowlist.
- Explicit cwd rules.
- Explicit env passthrough allowlist.
- Filesystem bounds enforcement using declared read/write paths.
- Required timeout.
- Direct exec only (no shell invocation, no shell expansion).

For MVP package validation, subprocess-capable packages must declare at least one filesystem bound (`permissions.filesystem.read` or `permissions.filesystem.write`).

## Configuration Resolution

Configuration precedence:

1. Command line flags.
2. Environment variables.
3. Project `myx.config.toml`.
4. Global user config.

Command-line values always win.

## Output Contract

All commands support:

- Human-readable default output.
- JSON output via `--json`.

JSON output requirements:

- Success output: top-level object with `command`, `ok`, and command-specific payload fields.
- Failure output: top-level object with `command`, `ok`, `timestamp`, and `error`.
- Failure error object: `error.code`, `error.message`, optional `error.details`.

## Exit Codes

- `0`: Success.
- `1`: Generic runtime or unexpected error.
- `2`: Invalid CLI usage.
- `3`: Validation error.
- `4`: Resolution/fetch error.
- `5`: Integrity/trust error.
- `6`: Policy denial.
- `7`: Required semantic mismatch during export/build.

## Lockfile Semantics

Install-mutating operations must:

1. Write `myx.lock` only after full success.
2. Preserve deterministic ordering of entries.
3. Record resolved source, version, digest, and install-time permission snapshot.
4. Use atomic write strategy (temporary file + rename).

## Determinism

Given identical package source, version, policy config, and target:

- `myx add` must produce identical lockfile state.
- `myx build` must produce identical exported artifacts.

If an adapter cannot preserve determinism for a field, it must emit a structured loss report and fail when the loss affects required semantics.
