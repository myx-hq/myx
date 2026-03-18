# Overview

myx is a package manager and compatibility layer for agent capabilities.

The MVP (v0) focuses on one deterministic loop:

1. initialize or obtain a package,
2. install with explicit policy review,
3. inspect identity/tools/permissions,
4. run tools with explicit policy/runtime enforcement,
5. build deterministic target artifacts.

## Why myx?

The agent ecosystem is fragmented: capability definitions and runtime expectations vary by platform. myx provides one canonical capability contract so package authors can describe a tool once and export it to supported runtimes.

For MVP, the goal is reliability and enforcement, not maximum ecosystem coverage.

## What Ships in MVP

- **Rust core + CLI** with `init`, `add`, `inspect`, `build`, `run`.
- **Capability Profile v1** with explicit `tool_class`, `execution`, and permissions.
- **Static index + local path resolution** (no hosted registry dependency).
- **Deterministic lockfile/install semantics** with integrity checks.
- **Built-in export targets**: `openai`, `mcp`, `skill`.
- **Global runtime executor** for declarative `http` and constrained `subprocess` actions.

## What Is Deferred

- Hosted registry and `myx publish` workflows.
- External adapter plugin model and dynamic adapter discovery.
- Tier-2 export targets (`vercel`, `claude`, `gemini`).

Deferred scope is tracked in RFC 0005.
