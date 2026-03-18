# Architecture

myx MVP is a Rust-core system centered on a canonical capability profile and deterministic build/runtime behavior.

## Runtime Layers (MVP)

1. **Package Sources**
- Local package path.
- Project-configured static indexes.

2. **Resolution + Validation**
- Resolve package source/version.
- Load `myx.yaml` and capability profile.
- Enforce schema/runtime validation before install/build.

3. **Policy + Install**
- Evaluate permissions against policy mode.
- Install into local store.
- Update `myx.lock` atomically on success only.

4. **Built-In Export Engine**
- Target selection by exact id: `openai`, `mcp`, `skill`.
- Deterministic artifact emission to `.myx/<target>/...`.
- Structured loss reporting with hard-fail on required mismatches.

5. **Execution Surface**
- Global runtime executor enforces declarative `http`/`subprocess` actions.
- MCP output uses generated launch artifacts (`run.sh` entrypoint) with strict protocol mode (`--protocol mcp`).

## Data Flow

```text
source (path/static index)
  -> resolve + validate
  -> policy decision
  -> store install + lockfile write
  -> build target artifacts
  -> runtime execution via executor/bridge
```

## MVP Boundary

MVP does **not** include:

- hosted registry-backed publish/install workflows,
- external adapter plugins,
- dynamic adapter discovery.

Those are post-MVP items tracked in RFC 0005.

## Design Rationale

The architecture deliberately favors deterministic behavior and enforcement-grade policy/runtime guarantees over broad first-release target count. Once the core loop is stable, post-MVP expansion reuses the same profile and loss-report contracts.
