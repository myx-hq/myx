# Overview

This document provides a high‑level overview of **myx**, a package manager and compatibility layer for agent capabilities.  myx is designed to bridge the gap between existing agent frameworks, such as MCP, SKILL.md, OpenAI tool schemas and bespoke tooling.  Rather than replacing existing standards, myx wraps them in a consistent package format, enabling installation, inspection and re‑export to different runtimes.

## Why myx?

The modern agent ecosystem is fragmented.  Capabilities live in many formats — MCP servers, SKILL.md bundles, tool definitions in JSON, framework plugins and one‑off scripts.  Each requires different installation steps and often cannot be easily reused across systems.  This fragmentation slows innovation and makes it hard for developers to share work.

myx introduces a canonical **Capability IR** (intermediate representation) that normalises disparate capability descriptions.  Using a set of **import adapters**, myx can ingest existing capability definitions into the IR.  **Export adapters** then emit runtime‑specific artifacts (e.g. OpenAI tool schemas, SKILL.md bundles or MCP server configurations).  This architecture lets developers:

- Package a capability once and reuse it across multiple runtimes.
- Inspect the permissions and behaviour of a capability before installation.
- Maintain reproducible installations via lockfiles and checksums.

## Components

The system comprises several key pieces:

- **Capability IR** — a canonical JSON schema that captures a capability’s identity, instructions, tools, permissions, runtime entrypoints and compatibility hints.
- **Import Adapters** — code that converts external formats (e.g. SKILL.md folders, MCP server descriptors) into the IR.
- **Export Adapters** — code that converts the IR into runtime‑specific artifacts (e.g. OpenAI tool definitions).
- **CLI** — a command line tool for developers to initialise packages, inspect them, install them, build runtime artifacts and publish to a registry.
- **Registry** — a service that hosts versioned packages, enabling search, download and publishing workflows.

The rest of the documentation delves into each of these components in more detail.