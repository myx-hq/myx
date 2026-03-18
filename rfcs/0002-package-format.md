# RFC 0002: Package Format (MVP)

*Status: Accepted*

## Summary

This RFC defines the on-disk package format consumed by the MVP Rust CLI.

A package must contain:

- `myx.yaml` (manifest)
- `capability.json` (Capability Profile v1)

Optional authoring assets (`prompts/`, `tools/`, `runtime/`) may exist, but exported target artifacts are generated into the consumer workspace (`.myx/<target>/...`) rather than mutating package source.

## Motivation

MVP needs deterministic, inspectable package structure that supports:

- local path install
- static-index resolution
- reproducible validation and lockfile entries
- deterministic target exports

## Directory Layout

```text
my-capability/
├── myx.yaml
├── capability.json
├── prompts/            # optional
│   ├── system.md
│   └── usage.md
├── tools/              # optional
│   └── schema.json
├── runtime/            # optional package assets
└── ...
```

## Manifest (`myx.yaml`)

Required fields:

- `name`
- `version`

Recommended fields:

- `description`
- `publisher`
- `license`
- `ir` (defaults to `./capability.json`)

Validation requirements:

1. `myx.yaml` must parse as valid YAML.
2. IR path must resolve to an existing file.
3. Manifest `name` and `version` must match `capability.json` identity values.

## Capability Profile (`capability.json`)

`capability.json` must validate against Capability Profile v1 (`schema_version: "1"`) and include required per-tool fields:

- `tool_class`
- `execution`

For subprocess tools, profile and permissions must satisfy MVP subprocess constraints.

## Install and Build Semantics

### Install (`myx add`)

- Package may come from local path or static index entry.
- Validation occurs before lockfile mutation.
- On success, package is materialized in local store and recorded in `myx.lock`.
- On failure, `myx.lock` remains unchanged.

### Build (`myx build --target <openai|mcp|skill>`)

- Build reads package profile from source/store.
- Exported artifacts are generated under project `.myx/<target>/...`.
- Package source files are not rewritten by build.
- Loss report is emitted on lossy conversion; required mismatches hard-fail.

## Backwards Compatibility

- Unknown extra files are ignored.
- Removal/renaming of `myx.yaml` or `capability.json` is breaking.
- Breaking schema changes require a new profile schema version.
