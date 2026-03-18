# RFC 0007: Runtime Executor Spec

*Status: Draft*

## Purpose

The runtime executor executes capability tools deterministically while enforcing declared permissions.

## Suggested Crate

```text
crates/myx-runtime
```

## Responsibilities

- validate execution blocks
- enforce permissions and policy
- execute `http` actions
- execute `subprocess` actions
- capture output and diagnostics
- return structured results
- never invoke a shell implicitly

## Execution Kinds

Supported in MVP:

- `http`
- `subprocess`

## Rust-Oriented API Sketch

```rust
pub enum ExecutionRequest {
    Http(HttpRequest),
    Subprocess(SubprocessRequest),
}

pub enum ExecutionStatus {
    Success,
    Failed,
    TimedOut,
    Denied,
}

pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub output: Option<String>,
    pub error: Option<ExecutionError>,
    pub duration_ms: u64,
}
```

## HTTP Execution Spec

### Example

```json
{
  "type": "http",
  "method": "GET",
  "url": "https://api.github.com/repos/{owner}/{repo}",
  "headers": {
    "Authorization": "Bearer {{GITHUB_TOKEN}}"
  },
  "timeout_ms": 5000
}
```

### Rules

- URL host must match `permissions.network`
- timeout is required
- redirects must not escape allowed hosts
- headers may only resolve declared secrets
- method must be explicit
- request body templates must validate before execution

## Subprocess Execution Spec

### Example

```json
{
  "type": "subprocess",
  "command": "git",
  "args": ["clone", "{{repo}}"],
  "cwd": "./workspace",
  "env": {
    "GIT_TOKEN": "{{GIT_TOKEN}}"
  },
  "timeout_ms": 10000
}
```

### Rules

- command must match exact allowlist
- no shell invocation
- no shell expansion
- no string command mode
- cwd must resolve inside allowed path bounds
- env passthrough must be explicitly allowlisted
- timeout is required
- stdout/stderr should be captured deterministically

## Execution Flow

```text
load tool
-> validate execution block
-> resolve templates
-> enforce policy
-> execute
-> capture result
-> return structured output
```

## Failure Modes

Runtime failures should be categorized with stable codes:

- `execution_invalid`
- `policy_denied`
- `network_denied`
- `subprocess_denied`
- `filesystem_denied`
- `timeout`
- `runtime_failure`

## Determinism Requirements

- same input + same environment policy should produce the same execution plan
- timeouts should be measured consistently
- template resolution should be explicit and deterministic
- failures should map to stable error codes and deterministic CLI exit codes
