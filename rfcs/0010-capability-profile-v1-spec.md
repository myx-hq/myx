# RFC 0010: Capability Profile v1 Spec

*Status: Draft*

## Purpose

Capability Profile v1 is the canonical contract for myx packages and exports.

All Tier-1 targets build from this profile.

## Required Top-Level Fields

- `name`
- `version`
- `description`
- `tools`
- `permissions`

## Tool Requirements

Each tool must include:

- `name`
- `description`
- `tool_class`
- `parameters`
- `execution`

## Tool Classes

Required enum:

- `http_api`
- `local_process`
- `filesystem_assisted`
- `composite`

### MVP Note

`composite` may remain reserved or minimally defined if not implemented in MVP.

## Execution Block

Required for every tool.

### HTTP Example

```json
{
  "type": "http",
  "method": "GET",
  "url": "https://api.github.com/search/repositories?q={{query}}",
  "headers": {
    "Authorization": "Bearer {{GITHUB_TOKEN}}"
  },
  "timeout_ms": 5000
}
```

### Subprocess Example

```json
{
  "type": "subprocess",
  "command": "git",
  "args": ["status"],
  "cwd": "./workspace",
  "env": {},
  "timeout_ms": 5000
}
```

## Permissions

### Required domains

- `network`
- `subprocess`
- `filesystem`

### Semantics

- `permissions.network` is a host allowlist
- `permissions.subprocess` is a structured command allowlist
- `permissions.filesystem` specifies readable and writable bounds

## Validation Rules

- `tool_class` required
- `execution` required
- timeout required
- permissions required for executable tools
- invalid or missing execution declarations fail deterministically

## Export Principle

Exports may be lossy, but:

- required mismatches hard-fail
- lossy conversions emit explicit loss reports
