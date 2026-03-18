# RFC 0011: Lockfile Spec

*Status: Draft*

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
  "version": 1,
  "packages": {
    "github": {
      "version": "0.1.0",
      "resolved": "https://index.example/github-0.1.0.tar.gz",
      "digest": "sha256:abc123",
      "permissions_snapshot": {
        "network": ["api.github.com"],
        "subprocess": ["git"]
      }
    }
  }
}
```

## Rules

- deterministic ordering
- atomic write
- no mutation on failed install
- failed `add` leaves previous lockfile unchanged
- digest must match fetched artifact

## Failure Modes

- `E_LOCK_WRITE`
- `E_LOCK_ATOMICITY`
- `E_DIGEST_MISMATCH`
