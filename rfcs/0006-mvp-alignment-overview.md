# RFC 0006: MVP Alignment Overview

*Status: Draft*

## Goal

Define what “all green” MVP alignment means for myx:

- clear spec coverage
- schema/type contracts where needed
- deterministic behavior
- explicit failure modes
- reference command flow

## MVP Definition

### Platform

- macOS-first
- Homebrew distribution target
- Rust-first implementation

### CLI Surface

- `myx init`
- `myx add <name|path>`
- `myx inspect <name|path>`
- `myx build --target <openai|mcp|skill>`
- `myx run <package>.<tool> [--input json]`

### Tier-1 Targets

- `openai`
- `mcp`
- `skill`

### Non-Goals for MVP

- hosted registry
- external plugin system
- Tier-2 targets (`vercel`, `claude`, `gemini`)
- package signing verification as a release gate
- broad subprocess flexibility
- daemonized hosted runtime architecture

## Core Principle

myx should execute as a deterministic pipeline:

```text
resolve -> validate -> inspect policy -> install -> run -> export
```

## Required Green Areas

- runtime executor
- policy enforcement
- run command
- capability profile enforcement
- lockfile semantics
- static index semantics
- Tier-1 export determinism
