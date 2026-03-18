# RFC 0013: MVP Green Checklist

*Status: Draft*

## Core Runtime

- [ ] Runtime executor spec implemented
- [ ] HTTP execution path implemented
- [ ] Subprocess execution path implemented
- [ ] No-shell enforcement implemented
- [ ] Policy enforcement wired into execution

## CLI

- [ ] `myx init`
- [ ] `myx add`
- [ ] `myx inspect`
- [ ] `myx build`
- [ ] `myx run`

## Contracts

- [ ] Capability Profile v1 enforced
- [ ] Lockfile deterministic and atomic
- [ ] Static index schema enforced
- [ ] Stable error codes implemented

## Tier-1 Targets

- [ ] `openai` export deterministic
- [ ] `mcp` wrapper generated and runnable
- [ ] `skill` export deterministic
- [ ] Loss reports emitted when lossy

## Validation and Tests

- [ ] Schema validation tests
- [ ] Resolver tests
- [ ] Policy denial tests
- [ ] Runtime executor tests
- [ ] Golden export tests
- [ ] MCP runnable check
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
