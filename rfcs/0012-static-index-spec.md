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
  "packages": {
    "github": [
      {
        "version": "0.1.0",
        "url": "https://example.com/github-0.1.0.tar.gz",
        "digest": "sha256:abc123",
        "description": "GitHub capability package",
        "capabilities": ["search_repos", "read_issues"]
      }
    ]
  }
}
```

## Requirements

- exact version resolution must be deterministic
- digest is required
- install source must be explicit
- precedence rules between indexes must be defined and deterministic

## Resolver Behavior

```text
load configured indexes
-> merge by precedence
-> resolve exact package version
-> fetch artifact
-> verify digest
-> install
```

## Failure Modes

- `E_RESOLVE`
- `E_VERSION_NOT_FOUND`
- `E_INDEX_INVALID`
- `E_DIGEST_MISMATCH`
