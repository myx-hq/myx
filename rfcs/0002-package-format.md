# RFC 0002: Package Format

*Status: Draft*

## Summary

This RFC defines the on‑disk format for an myx capability package.  A package consists of a directory containing metadata, the canonical capability IR, optional source files (e.g. prompts, tools) and adapter outputs.  Packages are archived into tarballs for distribution.

## Motivation

To publish and install capabilities reliably, we need a standard layout and manifest file.  The package format should be simple, composable and amenable to tooling (e.g. zip/tar operations).  It should also accommodate additional files without breaking consumers.

## Specification

An myx package is a directory with the following structure:

```
my-capability/
├── myx.yaml           # Package manifest (YAML)
├── capability.json       # Canonical Capability IR (JSON)
├── prompts/             # Prompt files (optional)
│   ├── system.md
│   └── usage.md
├── tools/               # Tool definitions (optional)
│   └── schema.json
├── adapters/            # Export artefacts (optional)
│   ├── openai/
│   │   ├── <name>.tools.json
│   │   └── <name>.instructions.md
│   ├── skill/
│   │   └── SKILL.md
│   └── mcp/
│       └── server.json
├── runtime/             # Runtime scripts or servers (optional)
│   └── mcp/server.js
└── ...                  # Additional package data
```

Key components:

- **myx.yaml** — A YAML manifest containing the package name, version, description, author and pointers to the IR file.  It may also specify which import/export adapters to use.
- **capability.json** — The canonical IR.  This file should always be present in packaged capabilities.
- **prompts/** — Contains Markdown prompt fragments referenced in the IR’s instructions section.  While the IR stores the text inline, packaging prompts separately allows for richer editing workflows.
- **tools/** — Contains JSON Schema fragments for tools.  Like prompts, these may be embedded in the IR but can be kept separate for clarity.
- **adapters/** — Stores prebuilt export outputs.  For example, after running `myx build --target openai`, the CLI writes the OpenAI artefacts here so that published packages already contain them.
- **runtime/** — Contains any code required to run long‑lived parts of the capability (e.g. an MCP server).  The IR references these via relative paths.

## Packaging Process

When publishing a package, the following steps occur:

1. Validate that `myx.yaml` and `capability.json` exist and are consistent.
2. Ensure the version in `myx.yaml` matches the version in the IR.
3. Run any configured build steps (e.g. run export adapters if artefacts are missing).
4. Tar and gzip the package directory into `<name>-<version>.myx.tgz`.
5. Compute a checksum and attach it to the package metadata.
6. Upload the tarball and metadata to the registry.

## Backwards Compatibility

The package format may evolve over time.  New files or directories can be added without breaking existing consumers.  The CLI should ignore unknown files.  Removal or renaming of the manifest or IR files constitutes a breaking change and will require a new major version of myx.