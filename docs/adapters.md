# Adapters

In MVP, adapters are built into the Rust CLI/core and are not loaded dynamically.

## MVP Adapter Model

- Target handlers are selected by exact `--target` value.
- Supported export targets: `openai`, `mcp`, `skill`.
- Adapters run only after profile/execution validation succeeds.
- Output is deterministic and written under `.myx/<target>/...`.
- Semantic loss is reported via structured `loss-report.json`; required mismatches fail build.

This model is intentional for v0 and keeps policy/runtime guarantees centralized.

## Built-In Export Targets

### OpenAI

Writes:

- `tools.json`
- `instructions.md`

### SKILL

Writes:

- `SKILL.md`

### MCP

Writes:

- `server.json`
- `runtime-config.json`
- `launch.json`
- `run.sh`

`run.sh` is the user-facing entrypoint. The underlying runtime bridge binary is an internal detail.

Generated MCP launch uses strict protocol mode:

- `--protocol mcp` with `Content-Length` framed JSON-RPC over stdio.

## Importers in MVP

Importer crates may exist and evolve during MVP, but importer completeness is not a release gate for v0.

## Post-MVP Expansion

The following are deferred to RFC 0005:

- external adapter/plugin loading,
- adapter discovery/listing commands,
- broader adapter conformance levels for third-party implementations.
