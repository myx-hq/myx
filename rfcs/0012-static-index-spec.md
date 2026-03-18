# RFC 0012: Static Index Spec

*Status: Draft*

## Purpose

Static indexes are the package discovery mechanism for MVP.

No hosted registry is required.

## Transport

- local file
- HTTP JSON document

## Example Schema

```json
{
  "schema_version": 1,
  "packages": [
    {
      "name": "github",
      "version": "0.1.0",
      "source": "/abs/path/to/github-0.1.0",
      "digest": "sha256:abc123"
    }
  ]
}
```

## Requirements

- exact version resolution must be deterministic
- digest is required
- install source must be explicit
- precedence rules between indexes must be defined and deterministic

## Deterministic Precedence

Resolver selection for `myx add <name>`:

1. highest semver version wins
2. for equal version matches, earlier `index.sources` entry in config wins

This precedence contract is deterministic and testable.

## Resolver Behavior

```text
load configured indexes
-> parse current v1 or migrate legacy shape
-> collect candidates by package name (and optional requested version)
-> pick highest version
-> tie-break equal version by index source order
-> return resolved local source path + digest
```

## Migration

Legacy index map shape remains readable for migration:

```json
{
  "packages": {
    "github": [
      {
        "version": "0.1.0",
        "url": "/abs/path/to/github-0.1.0",
        "digest": "sha256:abc123"
      }
    ]
  }
}
```

Migration behavior:

- legacy entries are converted to canonical entries (`name`, `version`, `source`, `digest`)
- `url`/`resolved` are accepted as legacy source aliases
- write/publish side remains canonical v1 shape

## Failure Modes

- `E_INDEX_INVALID` (invalid JSON or unsupported index shape)
- `E_VERSION_NOT_FOUND`
- `E_DIGEST_MISMATCH`
