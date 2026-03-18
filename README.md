# myx

**myx** is a package manager and compatibility layer for agent capabilities. It aims to be the missing packaging layer in the modern agent ecosystem, enabling developers to package, inspect, and install capabilities across runtimes without rewriting them.

## Goals

- **Package once, run anywhere.** myx normalizes disparate capability formats into a canonical model and exports them to target runtimes.
- **Inspect and trust.** Every package declares permissions (network, secrets, filesystem, subprocess) before install.
- **Reproducible installs.** Lockfile + digest checks provide deterministic environments.
- **Ecosystem compatibility.** myx is a bridge across agent ecosystems, not a replacement for them.

## Status

Early MVP implementation has started as a Rust monorepo. Specs and RFCs remain the source of truth while core crates are scaffolded for `init/add/inspect/build`.
Repository and org mapping guidance lives in `docs/repository-structure.md`.

## Rust Monorepo Layout

```text
myx/
├── Cargo.toml
├── rust-toolchain.toml
├── crates/
│   ├── myx-cli
│   ├── myx-core
│   ├── myx-resolver
│   ├── myx-store
│   ├── myx-lockfile
│   ├── myx-policy
│   ├── myx-runtime-executor
│   ├── adapter-export-openai
│   ├── adapter-export-mcp
│   ├── adapter-export-skill
│   ├── adapter-import-openai
│   └── adapter-import-skill
├── schemas/
│   ├── capability-profile/v1/schema.json
│   ├── index/v1/schema.json
│   └── loss-report/v1/schema.json
├── docs/
├── rfcs/
└── examples/
```

## Quickstart

```bash
cargo run -p myx-cli -- --help
```

## License

This project is licensed under **GNU AGPL v3.0** (`AGPL-3.0-only`).
