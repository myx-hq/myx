# Capability Profile (IR) v1

The myx Capability Profile is the canonical runtime-agnostic model used by the MVP CLI and adapters. This document defines the v1 shape used by `myx init/add/inspect/build`.

## Versioning

- `schema_version` is required.
- MVP expects `schema_version: "1"`.
- Breaking profile changes require a new major schema version.

## Top-Level Structure

A profile is a JSON object with these top-level fields:

- `schema_version` (string, required)
- `identity` (object, required)
- `metadata` (object, optional)
- `capabilities` (array of strings, optional)
- `instructions` (object, optional)
- `tools` (array, required, non-empty)
- `permissions` (object, required)
- `compatibility` (object, optional)

## Identity

- `identity.name` (string, required)
- `identity.version` (string, required)
- `identity.publisher` (string, optional)
- `identity.license` (string, optional, SPDX recommended)

## Tools (Required Contract)

Each tool requires:

- `name` (string)
- `description` (string)
- `parameters` (JSON Schema object)
- `tool_class` (enum, required)
  - `http_api`
  - `local_process`
  - `filesystem_assisted`
  - `composite`
- `execution` (object, required)

### `execution.kind = "http"`

Fields:

- `method` (string, required)
- `url` (string, required)
- `headers` (map, optional)
- `timeout_ms` (integer, optional)

### `execution.kind = "subprocess"`

Fields:

- `command` (string, required, executable token only)
- `args` (array of strings, optional)
- `cwd` (string, optional)
- `env_passthrough` (array of strings, optional)
- `timeout_ms` (integer, required in MVP)

## Permissions (Enforcement-Grade)

- `network` (array of hostnames)
- `secrets` (array of secret names)
- `filesystem` (object)
  - `read` (array of path rules)
  - `write` (array of path rules)
- `subprocess` (object)
  - `allowed_commands` (array)
  - `allowed_cwds` (array)
  - `allowed_env` (array)
  - `max_timeout_ms` (integer)

For subprocess-capable tools, subprocess permission fields must be declared and align with tool execution behavior.

## Compatibility

- `compatibility.runtimes` (array of target runtime ids)
- `compatibility.platforms` (array, optional)

MVP Tier-1 export targets are:

- `openai`
- `mcp`
- `skill`

## Minimal Example

```json
{
  "schema_version": "1",
  "identity": {
    "name": "github",
    "version": "0.1.0",
    "publisher": "myx-official",
    "license": "Apache-2.0"
  },
  "tools": [
    {
      "name": "search_repositories",
      "description": "Search GitHub repositories.",
      "parameters": {
        "type": "object",
        "properties": {
          "query": { "type": "string" }
        },
        "required": ["query"]
      },
      "tool_class": "http_api",
      "execution": {
        "kind": "http",
        "method": "GET",
        "url": "https://api.github.com/search/repositories?q={{query}}",
        "timeout_ms": 10000
      }
    }
  ],
  "permissions": {
    "network": ["api.github.com"],
    "secrets": ["GITHUB_TOKEN"],
    "filesystem": {
      "read": [],
      "write": []
    },
    "subprocess": {
      "allowed_commands": [],
      "allowed_cwds": [],
      "allowed_env": [],
      "max_timeout_ms": 10000
    }
  },
  "compatibility": {
    "runtimes": ["openai", "mcp", "skill"],
    "platforms": ["darwin", "linux"]
  }
}
```
