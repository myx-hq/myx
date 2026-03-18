# RFC 0013: MVP Green Checklist

*Status: Accepted*

## Core Runtime

- [x] Runtime executor spec implemented
- [x] HTTP execution path implemented
- [x] Subprocess execution path implemented
- [x] No-shell enforcement implemented
- [x] Policy enforcement wired into execution

## CLI

- [x] `myx init`
- [x] `myx add`
- [x] `myx inspect`
- [x] `myx build`
- [x] `myx run`

## Contracts

- [x] Capability Profile v1 enforced
- [x] Lockfile deterministic and atomic
- [x] Static index schema enforced
- [x] Stable error codes implemented

## Tier-1 Targets

- [x] `openai` export deterministic
- [x] `mcp` runtime runner generated and runnable
- [x] `skill` export deterministic
- [x] Loss reports emitted when lossy

## Validation and Tests

- [x] Schema validation tests
- [x] Resolver tests
- [x] Policy denial tests
- [x] Runtime executor tests
- [x] Golden export tests
- [x] MCP runnable check
- [ ] Warm-cache benchmark tracked

## Ship Gate

A release-ready MVP must satisfy:

```text
add -> inspect -> run -> build
```

with:

- deterministic behavior
- policy correctness
- Tier-1 target support
- explicit loss reporting
