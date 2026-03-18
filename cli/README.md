# CLI

This directory will contain the implementation of the myx command line interface.  The CLI is the primary user interface to the myx system.  It will be written in TypeScript and compiled to Node.js for cross‑platform distribution.

## Planned Commands

- `myx init` — initialise a new capability package scaffold in the current directory.
- `myx add <package>` — install a package from a local path or from the registry.  Supports version specifiers.
- `myx list` — list installed packages.
- `myx inspect <package>` — display information about an installed package (identity, description, permissions, available adapters).
- `myx build --target <adapter>` — build export artefacts for installed packages for a specified runtime.
- `myx publish` — publish the current package to the configured registry.

## Development

The CLI will depend on Node.js and the TypeScript compiler.  A `package.json` with appropriate scripts will be added once development begins.  In the meantime, refer to `docs/roadmap.md` for progress updates.