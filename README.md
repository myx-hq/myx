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

```bash
# build MCP artifacts for a package
cargo run -p myx-cli -- build --target mcp

# run generated MCP server artifact (runtime bridge is internal)
./.myx/mcp/run.sh --healthcheck
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development rules, required checks, and PR expectations.

## License

This project is licensed under **Apache License 2.0** (`Apache-2.0`).
