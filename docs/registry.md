# Package Registry

Status: Post-MVP design scope (not shipped in MVP v0).

Current MVP package discovery uses local paths and project-configured static indexes. Hosted registry workflows and `myx publish` are deferred (RFC 0005).

The content below describes the intended future registry direction.

## Objectives

The registry design focuses on the following goals:

1. **Versioned storage** — Each package version is immutable.
2. **Search and discovery** — Developers can search by name, description, and capabilities.
3. **Metadata and permissions** — Registry stores package identity, profile metadata, and package artifact.
4. **Trust and signatures** — Registry can surface signature/provenance signals.
5. **Open ecosystem** — API should support mirrors and private registries.

## Proposed API Endpoints (Post-MVP)

| Endpoint | Description |
|---------|-------------|
| `GET /search?q=<query>` | Search package summaries. |
| `GET /packages/<name>` | List package versions and metadata. |
| `GET /packages/<name>/versions/<version>` | Return metadata for one version. |
| `GET /packages/<name>/versions/<version>/download` | Download package artifact. |
| `POST /publish` | Publish a new package version (auth required). |

## Planned CLI Interaction (Post-MVP)

When registry support lands, `myx add` by package name will resolve against configured registries, download metadata/artifacts, validate integrity, and update local lockfile state.

`myx publish` remains deferred until hosted registry and auth contracts are finalized.
