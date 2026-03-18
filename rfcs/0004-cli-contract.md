# RFC 0004: CLI Contract and Package Lifecycle

*Status: Draft*

## Summary

This RFC defines the normative behavior of the myx CLI for v0.2:

- Command surface and semantics for `init`, `add`, `inspect`, `build`, `publish`, and `list-adapters`.
- Output and exit code contract for interactive and CI usage.
- Lockfile update semantics and deterministic resolution expectations.

The goal is to make CLI behavior predictable before implementation.

## Motivation

Current docs list planned commands, but there is no normative contract for:

- Inputs and defaults.
- How commands behave in non-interactive environments.
- Which failures are warnings vs hard errors.
- When and how lockfile state changes.

Without a CLI contract, adapters and registry work cannot converge on stable integration behavior.

## Goals

1. Define a minimal, coherent CLI surface for v0.2.
2. Ensure deterministic outcomes for install/build workflows.
3. Support both human-readable and machine-readable outputs.
4. Keep room for future extension without breaking scripts.

## Non-Goals

- Plugin runtime architecture details.
- UX copy and colorized terminal formatting.
- Full dependency solver design beyond direct package pins for v0.2.

## Command Contracts

### `myx init`

Purpose:
- Scaffold a package directory with baseline files.

Inputs:
- Optional target directory (default: current directory).
- Optional flags for package name, publisher, and template.

Behavior:
1. If target directory is empty or does not exist, create scaffold files: `myx.yaml`, `capability.json`, `prompts/system.md`, and `tools/` (empty or sample schema).
2. If required files already exist, command fails unless `--force` is set.
3. Scaffold values must be internally consistent (`name`, `version`, IR pointers).

### `myx add <name@version | path>`

Purpose:
- Add a package from registry or local path into the project.

Behavior:
1. Resolve source. Local path sources validate immediately; registry references fetch metadata and then tarball content.
2. Validate package structure and checksums.
3. Evaluate declared permissions against local policy.
4. Install package into local cache/workspace layout.
5. Update lockfile atomically on success.

Failure rules:
- Validation, integrity, or policy failure results in no lockfile mutation.
- Partial downloads must not leave active installed state.

### `myx inspect <name | path>`

Purpose:
- Show package metadata, compatibility, tools, and permissions.

Behavior:
- By default prints human-readable summary.
- `--json` prints structured output with stable keys.
- Missing package returns non-zero exit code.

### `myx build --target <adapter>`

Purpose:
- Produce runtime artifacts from `capability.json`.

Behavior:
1. Load and validate package IR.
2. Resolve export adapter by exact target id.
3. Generate artifacts under `adapters/<target>/`.
4. Optionally fail on adapter warnings when `--strict` is enabled.

Rules:
- Exporters must not mutate source IR files.
- Repeated builds with same inputs must be deterministic.

### `myx publish`

Purpose:
- Publish package to configured registry.

Behavior:
1. Validate package and ensure required files exist.
2. Ensure adapter artifacts are present or build them.
3. Create versioned archive and checksum.
4. Upload metadata and archive.
5. Reject overwrite of existing version.

### `myx list-adapters`

Purpose:
- Enumerate discoverable import/export adapters.

Behavior:
- Outputs adapter id, type (`import` or `export`), version, and capability flags.
- `--json` output is stable for scripting.

## Output Contract

Each command supports:

- Human-readable default output.
- Machine-readable output via `--json`.

JSON output rules:

- UTF-8 JSON object at top level.
- Includes `command`, `ok`, and `timestamp`.
- On failure includes `error.code`, `error.message`, and optional `error.details`.

## Exit Codes

- `0`: Success.
- `1`: Generic runtime or unexpected error.
- `2`: Invalid CLI usage (arguments, flags, missing required inputs).
- `3`: Validation error (manifest/IR/package structure).
- `4`: Resolution or fetch error (registry/network/not found).
- `5`: Integrity or trust failure (checksum/signature mismatch).
- `6`: Policy denial (permissions rejected by policy).

## Lockfile Semantics

`myx add` and other install-mutating operations must:

1. Write lockfile only after successful validation and install.
2. Preserve deterministic ordering of package entries.
3. Record resolved source, version, digest, and install-time permissions snapshot.
4. Use atomic write (temp file + rename) to prevent partial lockfiles.

Lockfile schema details are deferred to RFC 0005, but these semantics are normative for CLI behavior.

## Configuration Resolution

For v0.2, CLI config is resolved in this order:

1. Command line flags.
2. Environment variables.
3. Project-level config file (if present).
4. Global user config.

Explicit command-line values always win.

## Determinism and Reproducibility

For a fixed package source, version, and policy configuration:

- `myx add` must produce the same lockfile state.
- `myx build --target X` must produce byte-stable outputs where adapter inputs are unchanged.

If complete determinism is impossible for an adapter, it must emit a warning with reason.

## Open Questions

- Should `myx add` support version ranges in v0.2 or require exact pins only?
- Should `myx publish` require signatures by default or keep signatures optional?
- Should `build` write artifacts in-place only, or also support an explicit output directory?
