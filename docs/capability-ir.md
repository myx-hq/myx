# Capability IR

The **Capability IR** (intermediate representation) is the heart of myx.  It captures all information needed to install, inspect and export a capability, independent of the runtime in which it will be executed.  This document describes the structure of the IR as of version 0.1.

## Schema

The IR is defined as a JSON object with the following top‑level fields:

- `schema_version` (string): The version of the IR schema.  For example `"0.1"`.
- `identity` (object): Basic identification of the capability.
  - `name` (string): A human‑readable identifier (e.g. "github").
  - `version` (string): A semantic version (e.g. "0.1.2").
  - `publisher` (string): Identifier of the package author or organisation.
  - `license` (string, optional): SPDX license identifier.
- `metadata` (object, optional): Additional descriptive fields.
  - `description` (string): A short description.
  - `homepage` (string, optional): URL for more information.
  - `source` (string, optional): Link to the upstream source code.
- `capabilities` (array of strings): A list of semantic capability names such as `"read_issues"`, `"send_email"`, `"query_database"`.  These names are used for search and dependency analysis; they do not directly correspond to tool names.
- `instructions` (object): Prompt snippets that instruct an agent how to use the capability.
  - `system` (string): A system‑level prompt describing high‑level rules or context.
  - `usage` (string, optional): A usage prompt instructing the model when to call the capability.
- `tools` (array of objects): Structured descriptions of callable actions.  Each tool has:
  - `name` (string): The function name exposed to the agent.
  - `description` (string): A natural language description.
  - `parameters` (object): A JSON Schema object describing expected arguments (`type`, `properties`, `required`).
- `permissions` (object): Declarative declaration of what the capability may access.
  - `network` (array of strings): Allowed hostnames (e.g. `api.github.com`).
  - `secrets` (array of strings): Names of required secret variables (e.g. `GITHUB_TOKEN`).
  - `filesystem` (array of strings): Paths (or patterns) the capability may read/write.
  - `subprocess` (boolean): Whether the capability spawns subprocesses.
- `runtime` (object, optional): Additional runtime entrypoints, used for capabilities that require a server or long‑running process.
  - `entrypoints` (object): A map from entrypoint name to relative path.  Example: `"mcp_server": "./runtime/mcp/server.js"`.
- `compatibility` (object): Hints about supported runtimes and platforms.
  - `runtimes` (array of strings): Supported adapter targets (e.g. `"mcp"`, `"openai"`, `"skillmd"`).
  - `platforms` (array of strings, optional): Supported OS/architecture combinations (e.g. `"darwin-arm64"`).

## Example

Below is a simplified example IR for a GitHub capability:

```json
{
  "schema_version": "0.1",
  "identity": {
    "name": "github",
    "version": "0.1.2",
    "publisher": "myx-official",
    "license": "MIT"
  },
  "metadata": {
    "description": "GitHub capability bundle for agents.",
    "homepage": "https://github.com/myx/github",
    "source": "https://github.com/myx/github"
  },
  "capabilities": ["search_repos", "read_issues", "open_pull_requests"],
  "instructions": {
    "system": "You can use the GitHub tools to read repositories and manage pull requests.",
    "usage": "Use these tools when the user asks about GitHub repositories."
  },
  "tools": [
    {
      "name": "search_repositories",
      "description": "Search GitHub repositories by query.",
      "parameters": {
        "type": "object",
        "properties": {
          "query": { "type": "string" }
        },
        "required": ["query"]
      }
    }
  ],
  "permissions": {
    "network": ["api.github.com"],
    "secrets": ["GITHUB_TOKEN"],
    "filesystem": [],
    "subprocess": false
  },
  "runtime": {
    "entrypoints": {
      "mcp_server": "./runtime/mcp/server.js"
    }
  },
  "compatibility": {
    "runtimes": ["mcp", "openai", "skillmd"],
    "platforms": ["darwin", "linux"]
  }
}
```

## Evolution

The IR is versioned.  Breaking changes will result in a bump to `schema_version`.  The myx CLI and adapters are expected to handle multiple IR versions, providing graceful upgrade paths where possible.