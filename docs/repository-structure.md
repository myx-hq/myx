# Repository Structure Strategy

This document defines how `myx` is organized for MVP and how it should evolve beyond MVP.

## GitHub Organization

Recommended initial organization:

- `myx-hq/myx` - main Rust monorepo (core, CLI, adapters, schemas, RFCs, examples)
- `myx-hq/homebrew-myx` - Homebrew tap and formula updates
- `myx-hq/myx-index` - static package index data and schema validation

Optional later:

- `myx-hq/myx-registry` - hosted registry service
- `myx-hq/myx-conformance` - adapter conformance suite and fixtures
- `myx-hq/myx-official-packages` - official maintained package set

## Main Monorepo Responsibilities

`myx-hq/myx` owns:

- Rust CLI and core execution engine
- static index resolver
- lockfile and policy enforcement
- export adapters (MVP tier-1: openai, mcp, skill)
- import adapters (non-blocking for MVP)
- JSON Schemas for profile/index/loss report
- specs, RFCs, and examples

## Split Triggers

Keep one monorepo until at least one condition is true:

1. Hosted registry requires independent release cadence and operations.
2. Adapter ecosystem needs independent community contribution workflows.
3. CI duration and release coupling materially slows core delivery.

## Team Ownership (Suggested)

- Core/runtime/policy: `@myx-hq/core`
- Adapters and conversions: `@myx-hq/adapters`
- Specs and schemas: `@myx-hq/specs`
- Release and distribution: `@myx-hq/release`

When the repo is moved to GitHub, add CODEOWNERS to enforce this ownership model.
