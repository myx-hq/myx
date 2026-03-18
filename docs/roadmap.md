# Roadmap

This roadmap outlines the planned milestones for myx.  The versions and dates are aspirational and may change based on community feedback and available resources.

## v0.1 – Specification and Examples

*Status:* In progress

- Define the Capability IR schema (see `docs/capability-ir.md`).
- Specify import and export adapter interfaces.
- Draft RFCs for the capability model, package format and adapter API.
- Draft CLI behavior contract RFC to unblock implementation.
- Create example capability packages in the new package format (GitHub first, additional examples to follow).
- Publish documentation describing the vision, architecture and roadmap.

## v0.2 – CLI Prototype

*Goals:*

- Implement a Node.js/TypeScript CLI prototype supporting:
  - `myx init` – initialise a new package scaffold.
  - `myx add` – install a package from a local directory or registry.
  - `myx inspect` – display package metadata and permissions.
  - `myx build` – export the installed package to a runtime format (e.g. OpenAI tools, SKILL.md).
  - `myx publish` – publish a package to a registry.
- Implement importers for SKILL.md and MCP server descriptors.
- Implement exporters for OpenAI tool schemas and SKILL.md.
- Support basic lockfile creation and package caching.

## v0.3 – Registry Implementation

*Goals:*

- Build a minimal HTTP registry backing the CLI.
- Support package versioning, downloads, search and publishing.
- Implement package integrity checking via checksums.
- Provide configuration for custom registries and mirrors.

## v0.4 – Security and Signatures

*Goals:*

- Add optional package signing.  Packages can include a public key and signature in their metadata.
- CLI verifies signatures on installation and warns on mismatch.
- Integrate static analysis tools to detect obvious malware patterns.
- Expand permissions model with finer‑grained categories if needed.

## Beyond v1.0

Once myx reaches 1.0, the focus will shift to ecosystem growth:

- Expand adapter support to more formats (LangChain, Semantic Kernel, etc.).
- Foster a vibrant community of package authors and maintainers.
- Integrate with other agent infrastructure such as skill graphs and automated discovery.
- Experiment with sandbox enforcement to automatically restrict capabilities to their declared permissions.
