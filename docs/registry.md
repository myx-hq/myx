# Package Registry

The myx **registry** is a central repository where capability packages can be published and downloaded.  Similar to package registries such as npm, PyPI or crates.io, the myx registry stores versioned artefacts, metadata and security information.  The registry is not required to use myx locally, but it becomes essential for sharing packages across the community.

## Objectives

The registry design focuses on the following goals:

1. **Versioned storage** — Each package version is immutable.  Once published, a version cannot be overwritten; new versions must increment the version number.
2. **Search and discovery** — Developers should be able to search for capabilities by name, description, semantic tags and declared capabilities (e.g. `read_email`).  The registry may index capability names from the IR to improve discovery.
3. **Metadata and permissions** — The registry stores the package’s identity, metadata, capability IR and a copy of the package artefact (tarball).  Permissions are surfaced in search results and package pages to aid informed decision making.
4. **Trust and signatures** — Packages can optionally be signed by their publishers.  The registry may verify signatures and display trust badges.  Additional security scanning (e.g. static analysis for malicious code) is an area for future work.
5. **Open ecosystem** — The registry API should be open and accessible so that alternative registries, mirrors and private registries can be built.

## Basic API Endpoints

While the concrete API is subject to change, a minimal registry supports the following operations:

| Endpoint | Description |
|---------|-------------|
| `GET /search?q=<query>` | Search for packages by name, description, capabilities or tags.  Returns a list of matching package summaries. |
| `GET /packages/<name>` | Retrieve metadata about all versions of a package. |
| `GET /packages/<name>/versions/<version>` | Retrieve metadata about a specific version (including permissions, capabilities, etc.). |
| `GET /packages/<name>/versions/<version>/download` | Download the package tarball. |
| `POST /publish` | Publish a new package version.  Requires authentication. |

In addition, the registry may implement endpoints for verifying package signatures, listing publishers, rating packages and more.

## Registry and the CLI

The myx CLI interacts with the registry via simple HTTP requests.  When you run `myx add github@0.1.2`, the CLI will:

1. Resolve `github@0.1.2` against configured registries (by default, the official registry).
2. Download the metadata and tarball from the registry.
3. Verify the checksum and signature.
4. Unpack the package into the local cache and update the lockfile.

Similarly, `myx publish` will send the package tarball and metadata to the registry for storage.  The registry will reject duplicate versions and may require authentication tokens.