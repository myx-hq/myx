# Permissions Model (MVP)

myx treats permissions as enforceable install and runtime policy inputs, not just documentation.

## Policy Intent

The goal is deterministic security decisions:

- Same package + same policy config -> same allow/deny result.
- Non-interactive environments should never rely on implicit prompts.

## Permission Categories

## Network

`permissions.network` lists allowed hostnames.

- Prefer explicit hosts (`api.github.com`) over broad domains or wildcards.
- Runtime HTTP actions must stay within this allowlist.

## Secrets

`permissions.secrets` lists required secret identifiers.

- Keep this list minimal.
- Secret presence and mapping are policy/runtime concerns; declaration is mandatory for transparency.

## Filesystem

`permissions.filesystem` is structured:

- `read`: allowed read path rules
- `write`: allowed write path rules

Avoid broad top-level rules when narrower paths are sufficient.

## Subprocess

`permissions.subprocess` is structured and required for subprocess-capable tools:

- `allowed_commands`
- `allowed_cwds`
- `allowed_env`
- `max_timeout_ms`

MVP requires strict execution constraints:

- exact command allowlist
- explicit cwd allowlist
- explicit env passthrough allowlist
- required timeout
- direct exec only (no shell invocation, no shell expansion)

## Policy Modes

MVP supports:

- `review_required` (default)
- `permissive`
- `strict`

## Interactive Behavior

In `review_required`, if package permissions exceed configured allowlists, install requires explicit user approval.

## Non-interactive / CI Behavior

If permissions exceed configured allowlists, install is denied. No prompt fallback.

Non-interactive mode is detected deterministically with this precedence:

1. `--non-interactive` flag.
2. `MYX_NON_INTERACTIVE` env override (`1/0`, `true/false`, `yes/no`, `on/off`).
3. Truthy `CI` env.
4. Non-TTY stdio (`stdin` or `stdout` is not a terminal).

If `MYX_NON_INTERACTIVE` is present with an invalid value, install fails with a validation error.

## Best Practices

- Declare least privilege.
- Keep subprocess usage narrow and auditable.
- Treat policy changes as security-relevant and test-covered.
