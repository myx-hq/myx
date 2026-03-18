# MVP Hardening Audit (2026-03)

This audit captures architecture and implementation risks identified after the MVP build loop was implemented.

## Goal

Track and execute hardening tasks to ensure:

- deterministic install/build/runtime behavior
- enforcement-grade policy/runtime guarantees
- clean and consistent contracts across RFCs/docs/schemas/code

## Issue Backlog

1. `P0` Verify index digest over full package payload (not profile-only).
2. `P0` Make store install non-destructive and atomic.
3. `P1` Fix MCP wrapper cwd semantics for subprocess tools.
4. `P1` Align loss-report schema with emitted payload.
5. `P1` Strengthen capability profile schema to match runtime validation.
6. `P2` Add strict MCP protocol compatibility mode.
7. `P2` Improve non-interactive policy behavior defaults.
8. `P2` Refactor `myx-cli` monolith into module-based command handlers.
9. `P2` Align RFC 0003 and high-level docs to MVP architecture.

## Execution Order

1. P0 integrity/safety fixes
2. P1 contract alignment and runtime semantics
3. P2 architecture cleanup and documentation cohesion
