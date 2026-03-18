# Registry

This directory will house the implementation of the myx package registry service.  The registry enables developers to publish, search and download capability packages.  It will expose a REST API for consumption by the myx CLI and other tools.

## Planned Features

- Immutable versioned storage of capability packages.
- Search by name, description, capabilities and metadata.
- Upload authentication and authorisation for publishers.
- Package signature verification and security scanning (future work).
- Mirror configuration to support private and regional registries.

The registry implementation is not yet started.  See `docs/registry.md` and `docs/roadmap.md` for the current design and timeline.