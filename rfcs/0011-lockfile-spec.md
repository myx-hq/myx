# RFC 0011: Lockfile Spec

*Status: Accepted*

## File Name

```text
myx.lock
```

## Purpose

The lockfile captures deterministic install state.

## Required Data

For each installed package:

- resolved source
- version
- digest
- install-time permission snapshot

## Example

```json
{
  "lockfile_version": 1,
  "packages": [
    {
      "name": "github",
      "version": "0.1.0",
      "source": "/abs/path/to/github-0.1.0",
      "digest": "sha256:abc123",
      "permissions_snapshot": {
        "network": ["api.github.com"],
        "subprocess": {
          "allowed_commands": ["git"]
        }
      }
    }
  ]
}
```

## Rules

- deterministic ordering
- atomic write
- no mutation on failed install
- failed `add` leaves previous lockfile unchanged
- digest must match fetched artifact
- canonical v1 output uses `lockfile_version` + array `packages` entries sorted by `(name, version)`

## Migration

Legacy lockfiles with this historical shape are accepted for read-time migration:

```json
{
  "version": 1,
  "packages": {
    "github": {
      "version": "0.1.0",
      "resolved": "/abs/path/to/github-0.1.0",
      "digest": "sha256:abc123",
      "permissions_snapshot": {}
    }
  }
}
```

Migration behavior:

- loader converts legacy map entries to canonical array entries
- `resolved` maps to canonical `source`
- migrated in-memory entries are deterministically sorted by `(name, version)`
- writes always emit canonical v1 shape

## Failure Modes

- lockfile write failure
- atomic rename failure
- parse failure for unknown lockfile shape
