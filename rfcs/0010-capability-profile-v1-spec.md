# RFC 0010: Capability Profile v1 Spec

*Status: Accepted*

## Purpose

Capability Profile v1 is the canonical contract for myx packages and exports.

All Tier-1 targets build from this profile.

## Required Top-Level Fields

- `schema_version` (`"1"`)
- `identity`
- `tools`
- `permissions`

Top-level `identity` fields:

- required: `name`, `version`
- optional: `publisher`, `license`

Optional top-level fields:

- `metadata` (`description`, `homepage`, `source`)
- `capabilities`
- `instructions`
- `compatibility`

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
  "kind": "http",
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
  "kind": "subprocess",
  "command": "git",
  "args": ["status"],
  "cwd": "./workspace",
  "env_passthrough": ["HOME"],
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
- `schema_version` must be `"1"`
- timeout required for subprocess execution
- permissions required for executable tools
- invalid or missing execution declarations fail deterministically

## Migration Plan (Legacy -> v1)

Legacy profile drafts used flat top-level fields like:

- `name`
- `version`
- `description`

without the `identity` object.

Migration to v1:

1. Move `name` and `version` into `identity.name` and `identity.version`.
2. Move top-level `description` into `metadata.description`.
3. Ensure each execution block uses `kind` (`http` or `subprocess`).
4. Replace subprocess `env` shape with `env_passthrough` key list.

Compatibility behavior:

- MVP parser expects v1 shape and rejects legacy top-level identity format with a deterministic migration error.
- No implicit in-memory shape conversion is performed in MVP to avoid ambiguous behavior.

## Export Principle

Exports may be lossy, but:

- required mismatches hard-fail
- lossy conversions emit explicit loss reports
