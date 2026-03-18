# RFC 0008: Policy Enforcement Spec

*Status: Accepted*

## Purpose

Policy enforcement ensures tool execution adheres strictly to declared permissions and explicit operator approval.

Default stance:

- deny by default
- allow only explicitly declared and approved behavior

## Policy Model

```rust
pub struct Policy {
    pub network_hosts: Vec<String>,
    pub allowed_commands: Vec<String>,
    pub allowed_env: Vec<String>,
    pub filesystem_read: Vec<PathBuf>,
    pub filesystem_write: Vec<PathBuf>,
}
```

## Enforcement Domains

### Network

- only declared hosts are allowed
- matching must be exact (or explicitly pattern-based in future versions)
- redirects to undeclared hosts must fail

### Subprocess

- exact command allowlist only in MVP
- no shell
- no implicit shell wrappers
- cwd must be within allowed filesystem bounds
- env keys must be explicitly allowed

### Filesystem

- read bounds and write bounds must be checked separately
- subprocess actions inherit filesystem constraints
- path normalization must happen before comparison

## Review Modes

### Interactive Default

When a package is added or first run, the user is shown requested permissions and prompted to approve.

Example:

```text
This package requests:
- network: api.github.com
- subprocess: git
- filesystem write: ./workspace

Allow? (y/n)
```

### CI / Non-Interactive

- explicit allowlist required
- absence of allowlist is a hard failure
- no prompt fallback

## Failure Contract

### Example JSON

```json
{
  "error": "policy_denied",
  "reason": "command 'bash' not allowed"
}
```

## Enforcement Timing

Policy checks happen at:

- install review
- run-time execution
- build-time validation when required semantics imply execution capability constraints

## Recommended Exit/Error Categories

- `policy_denied`
- `policy_invalid_configuration`
- `policy_prompt_io`
- `network_denied`
- `subprocess_denied`
- `filesystem_denied`

## MVP Constraints

- no policy wildcards for subprocess commands
- no shell-based execution
- no implicit env inheritance beyond declared passthrough
